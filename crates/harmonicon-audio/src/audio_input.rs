// SPDX-License-Identifier: MIT

use bevy::log::{error, info, info_span};
use bevy::prelude::{Res, ResMut, Resource, World};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use crossbeam_channel::{Receiver, Sender, bounded};

use crate::AudioSettings;

// The analysis window itself belongs with the analysis: re-exported from
// `harmonicon-dsp` so an offline run (`harmonicon-bench`) chunks audio
// exactly as the live mic path does, without depending on cpal or Bevy to
// learn the numbers.
pub use harmonicon_dsp::{CHUNK_SIZE, HOP_SIZE};

/// How many chunk buffers circulate between the real-time audio callback and
/// the consumer (`process_audio`). Comfortably more than one in flight at a
/// time — the consumer drains far faster than chunks arrive (one FFT per
/// ~46ms chunk) — so recycling normally never runs dry; if it ever does
/// (startup, or the consumer briefly falling behind), `push_chunks` falls
/// back to allocating a fresh buffer rather than dropping audio.
const POOL_SIZE: usize = 8;

// NonSend resource — keeps the cpal stream alive for the duration of the app.
#[allow(dead_code)]
pub struct AudioStream(pub cpal::Stream);

#[derive(Resource)]
pub struct AudioCapture {
    pub receiver: Receiver<Vec<f32>>,
    /// Hand a chunk buffer back here once you're done with it (e.g. when
    /// overwriting `AudioFrame::samples` with a newer chunk) so the
    /// real-time callback can reuse it instead of allocating — see
    /// `push_chunks`. Calling into the allocator from that callback risks
    /// blocking on a lock held by a lower-priority thread, causing an
    /// audible dropout ("xrun") on weaker machines.
    pub free_sender: Sender<Vec<f32>>,
    pub sample_rate: u32,
    /// The actually-connected device's name — may differ from the requested
    /// one if it wasn't found and capture fell back to the system default.
    pub device_name: String,
    /// Stream errors reported by cpal *after* the stream opened successfully
    /// — above all, the device being unplugged mid-session.
    ///
    /// cpal delivers these on its own thread, so they can't touch the `World`
    /// directly; [`detect_stream_failure`] drains this every frame and turns
    /// the first one into [`MicStatus::Failed`]. Before this channel existed
    /// the callback only did an `eprintln!`, which meant a mic unplugged
    /// during a song left `MicStatus` reading `Connected` forever: the
    /// Options banner stayed hidden, the in-play warning never appeared, and
    /// the game just silently stopped scoring.
    pub errors: Receiver<String>,
}

/// Whether the microphone capture stream is currently up.
///
/// Written from two places, which between them cover both ways a mic can be
/// unusable: [`start_capture`] (startup, and the Options Retry button) for a
/// stream that won't open, and [`detect_stream_failure`] for one that opened
/// and later died. Read by the Options banner and the in-play warning
/// overlay, so neither has to guess whether the game can hear anything.
#[derive(Resource, Clone, PartialEq, Debug)]
pub enum MicStatus {
    Connected {
        device_name: String,
    },
    Failed {
        reason: String,
    },
    /// Android's `RECORD_AUDIO` and iOS's `NSMicrophoneUsageDescription`
    /// both require an explicit runtime permission prompt before capture
    /// can succeed — this is somewhere for that state to land distinct
    /// from a hard failure, so the Options page can show "waiting on
    /// permission" rather than a generic error. Set by `start_capture` when
    /// `permission::microphone_granted()` says no — which off Android is
    /// never, since there the check always answers "granted".
    AwaitingPermission,
}

impl MicStatus {
    /// Whether capture is actually running. Anything else means the game is
    /// deaf, and every screen that cares says so — the Options banner and
    /// the in-play warning overlay both branch on exactly this, so "what
    /// counts as working" has one definition rather than one per screen.
    pub fn is_connected(&self) -> bool {
        matches!(self, MicStatus::Connected { .. })
    }
}

/// Names of every input device the current host reports, in host-listed
/// order. Empty (rather than an error) if enumeration itself fails — callers
/// treat "no devices" and "enumeration failed" the same way.
pub fn input_device_names() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices
            .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Which device name to actually look for, given the user's configured
/// preference (`""` means "use the system default"). Returns `None` when
/// `wanted` is empty or doesn't match anything currently plugged in, so the
/// caller falls back to the default device instead of erroring — a saved
/// preference for a since-unplugged device shouldn't brick capture.
fn resolve_device_name(available: &[String], wanted: &str) -> Option<String> {
    if wanted.is_empty() {
        return None;
    }
    available.iter().find(|n| n.as_str() == wanted).cloned()
}

/// (Re)starts the microphone capture stream using `AudioSettings::input_device`
/// (falling back to the system default if that device is empty/unavailable),
/// and records the outcome in [`MicStatus`]. Only needs `&mut World`, so both
/// the startup system and the Options page's "Retry" button / device picker
/// can trigger it directly (the latter via `Commands::queue`).
pub fn start_capture(world: &mut World) {
    // On a platform with a runtime permission model, opening the stream
    // before the user has granted it fails in a way indistinguishable from a
    // broken device. Ask first, and park in `AwaitingPermission` until the
    // answer comes back (see `retry_capture_when_permission_granted`).
    if !crate::permission::microphone_granted() {
        crate::permission::request_microphone();
        world.insert_resource(MicStatus::AwaitingPermission);
        return;
    }

    let wanted = world.resource::<AudioSettings>().input_device.clone();
    // Skip enumeration entirely for the common case (no preference set) — on
    // Linux, listing input devices makes cpal probe every ALSA/JACK backend,
    // which is noisy and pointless when we're just taking the default anyway.
    let device_name = if wanted.is_empty() {
        None
    } else {
        resolve_device_name(&input_device_names(), &wanted)
    };

    match create_audio_capture(device_name.as_deref()) {
        Ok((stream, capture)) => {
            info!(
                "Audio capture started at {} Hz on \"{}\"",
                capture.sample_rate, capture.device_name
            );
            world.insert_resource(MicStatus::Connected {
                device_name: capture.device_name.clone(),
            });
            world.insert_non_send(stream);
            world.insert_resource(capture);
        }
        Err(e) => {
            error!("Failed to start audio capture: {e}");
            world.insert_resource(MicStatus::Failed {
                reason: e.to_string(),
            });
        }
    }
}

/// Turns a cpal stream error into [`MicStatus::Failed`], so a microphone
/// unplugged *mid-session* is reported the same way one that never opened is.
///
/// Startup failure was always handled — `start_capture` sets `Failed` when
/// the stream won't open. A device that dies *after* opening is a different
/// path entirely: cpal reports it through the stream's error callback on its
/// own thread, and that callback used to only `eprintln!`. So `MicStatus`
/// stayed `Connected`, the Options banner stayed hidden, the in-play warning
/// never fired, and the player just watched their notes stop scoring.
///
/// Deliberately does **not** try to reopen the stream. An automatic retry
/// would need a backoff (cpal errors arrive in bursts), and on a machine with
/// no working input at all it would probe the audio backend forever. Recovery
/// stays the explicit Retry button on the Options page, which already exists
/// and already re-runs `start_capture`.
pub fn detect_stream_failure(
    capture: Option<Res<AudioCapture>>,
    status: Option<ResMut<MicStatus>>,
) {
    let (Some(capture), Some(mut status)) = (capture, status) else {
        return;
    };
    // Drain rather than take one: a dying device reports repeatedly, and
    // anything left queued would re-trigger this on later frames.
    let mut first: Option<String> = None;
    while let Ok(reason) = capture.errors.try_recv() {
        first.get_or_insert(reason);
    }
    let Some(reason) = first else {
        return;
    };
    // Only downgrade from `Connected`. A stream error while parked in
    // `AwaitingPermission` is the *denial* showing up as an I/O failure on
    // Android — "grant the permission" stays the more useful thing to say —
    // and overwriting an existing `Failed` would just churn its reason.
    if status.is_connected() {
        error!("Audio stream failed: {reason}");
        *status = MicStatus::Failed { reason };
    }
}

/// Polls for the permission dialog being answered, then starts capture for
/// real.
///
/// Only does anything while [`MicStatus::AwaitingPermission`] is the current
/// status, which off Android is never — [`permission::microphone_granted`]
/// answers `true` there, so `start_capture` never parks and this system
/// returns on its first line forever.
///
/// A poll rather than a callback because the permission result is delivered
/// to the Java activity, not to us; see [`permission::request_microphone`].
/// If the user *denies* it, this simply keeps polling and the status stays
/// `AwaitingPermission` — which is the truth, and what the Options page
/// already renders a banner for.
pub fn retry_capture_when_permission_granted(world: &mut World) {
    if !matches!(
        world.get_resource::<MicStatus>(),
        Some(MicStatus::AwaitingPermission)
    ) {
        return;
    }
    if !crate::permission::microphone_granted() {
        return;
    }
    start_capture(world);
}

/// Opens capture on `device_name` (falling back to the system default if
/// `None` or not found among the current input devices).
pub fn create_audio_capture(
    device_name: Option<&str>,
) -> Result<(AudioStream, AudioCapture), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = device_name
        .and_then(|name| {
            host.input_devices().ok()?.find(|d| {
                d.description()
                    .map(|desc| desc.name() == name)
                    .unwrap_or(false)
            })
        })
        .or_else(|| host.default_input_device())
        .ok_or("no input device available")?;
    let device_name = device
        .description()
        .map(|desc| desc.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate();
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();

    println!("Input device : {device_name}");
    println!(
        "Sample rate  : {} Hz  |  channels: {}  |  format: {:?}",
        sample_rate, channels, sample_format
    );

    let (tx, rx) = bounded::<Vec<f32>>(64);

    // Small and bounded: only the first error actually matters (they arrive
    // in bursts once a device dies), and a full channel must never block
    // cpal's callback thread — `try_send` drops the surplus.
    let (err_tx, err_rx) = bounded::<String>(4);

    // Pre-warm the recycling pool so even the first few chunks don't need to
    // allocate — see `AudioCapture::free_sender` / `push_chunks`.
    let (free_tx, free_rx) = bounded::<Vec<f32>>(POOL_SIZE);
    for _ in 0..POOL_SIZE {
        let _ = free_tx.try_send(Vec::with_capacity(CHUNK_SIZE));
    }

    let stream = match sample_format {
        SampleFormat::F32 => {
            build_stream_f32(&device, &stream_config, channels, tx, free_rx, err_tx)?
        }
        SampleFormat::I16 => {
            build_stream_i16(&device, &stream_config, channels, tx, free_rx, err_tx)?
        }
        SampleFormat::I32 => {
            build_stream_i32(&device, &stream_config, channels, tx, free_rx, err_tx)?
        }
        fmt => return Err(format!("unsupported sample format: {fmt:?}").into()),
    };

    stream.play()?;

    Ok((
        AudioStream(stream),
        AudioCapture {
            receiver: rx,
            free_sender: free_tx,
            sample_rate,
            device_name,
            errors: err_rx,
        },
    ))
}

// ---------------------------------------------------------------------------
// Per-format stream builders — identical logic, only the sample type differs.
// ---------------------------------------------------------------------------

fn build_stream_f32(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: Sender<Vec<f32>>,
    free_rx: Receiver<Vec<f32>>,
    errors: Sender<String>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let mut buf: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);
    let mut mono: Vec<f32> = Vec::with_capacity(CHUNK_SIZE / 2);
    device.build_input_stream(
        config,
        move |data: &[f32], _| push_chunks(&mut buf, &mut mono, data, channels, &tx, &free_rx),
        // Runs on cpal's thread: `try_send` only, never a blocking send.
        move |e| {
            let _ = errors.try_send(e.to_string());
        },
        None,
    )
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: Sender<Vec<f32>>,
    free_rx: Receiver<Vec<f32>>,
    errors: Sender<String>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let mut buf: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);
    let mut mono: Vec<f32> = Vec::with_capacity(CHUNK_SIZE / 2);
    let mut converted: Vec<f32> = Vec::with_capacity(CHUNK_SIZE / 2);
    device.build_input_stream(
        config,
        move |data: &[i16], _| {
            converted.clear();
            converted.extend(data.iter().map(|&s| s as f32 / 32_768.0));
            push_chunks(&mut buf, &mut mono, &converted, channels, &tx, &free_rx);
        },
        // Runs on cpal's thread: `try_send` only, never a blocking send.
        move |e| {
            let _ = errors.try_send(e.to_string());
        },
        None,
    )
}

fn build_stream_i32(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: Sender<Vec<f32>>,
    free_rx: Receiver<Vec<f32>>,
    errors: Sender<String>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let mut buf: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);
    let mut mono: Vec<f32> = Vec::with_capacity(CHUNK_SIZE / 2);
    let mut converted: Vec<f32> = Vec::with_capacity(CHUNK_SIZE / 2);
    device.build_input_stream(
        config,
        move |data: &[i32], _| {
            converted.clear();
            converted.extend(data.iter().map(|&s| s as f32 / 2_147_483_648.0));
            push_chunks(&mut buf, &mut mono, &converted, channels, &tx, &free_rx);
        },
        // Runs on cpal's thread: `try_send` only, never a blocking send.
        move |e| {
            let _ = errors.try_send(e.to_string());
        },
        None,
    )
}

/// Downmixes multichannel interleaved frames to mono into the reusable
/// `mono` scratch buffer, accumulates into `buf`, and emits CHUNK_SIZE
/// blocks with 50% overlap. Every buffer here (`buf`, `mono`, and the chunk
/// handed to `tx`, drawn from `free_rx`) is reused across calls rather than
/// freshly allocated, since this runs on the real-time audio callback
/// thread — calling into the allocator there risks blocking on a lock held
/// by a lower-priority thread and causing an audible dropout.
fn push_chunks(
    buf: &mut Vec<f32>,
    mono: &mut Vec<f32>,
    data: &[f32],
    channels: usize,
    tx: &Sender<Vec<f32>>,
    free_rx: &Receiver<Vec<f32>>,
) {
    // This runs on cpal's real-time callback thread, invisible to Bevy's own
    // per-system spans (those only wrap systems the ECS schedule calls) — a
    // manual span here is the only way Tracy shows this thread's activity at
    // all, which matters since it's the one place an allocator stall would
    // cause an audible dropout rather than just a dropped frame.
    let _span = info_span!("push_chunks", frames = data.len()).entered();
    mono.clear();
    if channels == 1 {
        mono.extend_from_slice(data);
    } else {
        mono.extend(
            data.chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32),
        );
    }
    buf.extend_from_slice(mono);
    while buf.len() >= CHUNK_SIZE {
        // Reuse a buffer the consumer already handed back if one's
        // available; only allocate as a last resort (pool momentarily
        // empty), so steady-state operation never touches the allocator.
        let mut chunk = free_rx
            .try_recv()
            .unwrap_or_else(|_| Vec::with_capacity(CHUNK_SIZE));
        chunk.clear();
        chunk.extend_from_slice(&buf[..CHUNK_SIZE]);
        let _ = tx.try_send(chunk);
        buf.drain(..HOP_SIZE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_device_name ──────────────────────────────────────────────────

    #[test]
    fn empty_preference_means_use_the_default() {
        assert_eq!(resolve_device_name(&["Mic A".to_string()], ""), None);
    }

    #[test]
    fn finds_a_currently_available_match() {
        let available = vec!["Mic A".to_string(), "Mic B".to_string()];
        assert_eq!(
            resolve_device_name(&available, "Mic B"),
            Some("Mic B".to_string())
        );
    }

    #[test]
    fn falls_back_to_default_when_the_saved_device_is_unplugged() {
        let available = vec!["Mic A".to_string()];
        assert_eq!(resolve_device_name(&available, "USB Mic (unplugged)"), None);
    }

    // ── push_chunks ──────────────────────────────────────────────────────────

    #[test]
    fn emits_a_full_chunk_and_keeps_the_overlap_tail() {
        let (tx, rx) = bounded::<Vec<f32>>(4);
        let (_free_tx, free_rx) = bounded::<Vec<f32>>(4); // empty pool: falls back to alloc
        let mut buf = Vec::new();
        let mut mono = Vec::new();
        let data: Vec<f32> = (0..CHUNK_SIZE).map(|i| i as f32).collect();

        push_chunks(&mut buf, &mut mono, &data, 1, &tx, &free_rx);

        let chunk = rx.try_recv().expect("one chunk should have been emitted");
        assert_eq!(chunk, data);
        // 50% overlap: the back half stays buffered for the next call.
        assert_eq!(buf, &data[CHUNK_SIZE / 2..]);
        assert!(rx.try_recv().is_err(), "only one chunk should have emitted");
    }

    #[test]
    fn downmixes_multichannel_frames_by_averaging() {
        let (tx, rx) = bounded::<Vec<f32>>(4);
        let (_free_tx, free_rx) = bounded::<Vec<f32>>(4);
        let mut buf = Vec::new();
        let mut mono = Vec::new();
        // Two channels interleaved: (1,3) -> 2.0, (2,4) -> 3.0.
        let data = vec![1.0, 3.0, 2.0, 4.0];

        push_chunks(&mut buf, &mut mono, &data, 2, &tx, &free_rx);

        assert_eq!(buf, vec![2.0, 3.0]);
        assert!(
            rx.try_recv().is_err(),
            "not enough samples yet for a full chunk"
        );
    }

    #[test]
    fn reuses_a_recycled_buffer_instead_of_allocating() {
        let (tx, rx) = bounded::<Vec<f32>>(4);
        let (free_tx, free_rx) = bounded::<Vec<f32>>(4);

        let recycled: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);
        let recycled_ptr = recycled.as_ptr();
        free_tx.try_send(recycled).unwrap();

        let mut buf = Vec::new();
        let mut mono = Vec::new();
        let data = vec![0.0f32; CHUNK_SIZE];

        push_chunks(&mut buf, &mut mono, &data, 1, &tx, &free_rx);

        let chunk = rx.try_recv().expect("chunk emitted");
        assert_eq!(
            chunk.as_ptr(),
            recycled_ptr,
            "should reuse the pooled allocation instead of a fresh one"
        );
    }

    /// An `AudioCapture` whose only live wire is the error channel — the
    /// sample-path fields are real but unused here, since
    /// `detect_stream_failure` never touches them.
    fn capture_reporting(errors: Receiver<String>) -> AudioCapture {
        let (tx, rx) = bounded::<Vec<f32>>(1);
        AudioCapture {
            receiver: rx,
            free_sender: tx,
            sample_rate: 44_100,
            device_name: "Test Device".into(),
            errors,
        }
    }

    fn app_with(status: MicStatus, errors: Receiver<String>) -> bevy::prelude::App {
        let mut app = bevy::prelude::App::new();
        app.insert_resource(capture_reporting(errors))
            .insert_resource(status)
            .add_systems(bevy::prelude::Update, detect_stream_failure);
        app
    }

    #[test]
    fn a_device_dying_mid_session_downgrades_connected_to_failed() {
        // The whole point: cpal reports this *after* the stream opened, on
        // its own thread. Before the error channel existed the status stayed
        // Connected and the player just watched their notes stop scoring.
        let (tx, rx) = bounded::<String>(4);
        tx.try_send("device disconnected".into()).unwrap();
        let mut app = app_with(
            MicStatus::Connected {
                device_name: "Test Device".into(),
            },
            rx,
        );
        app.update();
        assert_eq!(
            *app.world().resource::<MicStatus>(),
            MicStatus::Failed {
                reason: "device disconnected".into()
            }
        );
    }

    #[test]
    fn a_healthy_stream_leaves_the_status_alone() {
        let (_tx, rx) = bounded::<String>(4);
        let connected = MicStatus::Connected {
            device_name: "Test Device".into(),
        };
        let mut app = app_with(connected.clone(), rx);
        app.update();
        assert_eq!(*app.world().resource::<MicStatus>(), connected);
    }

    #[test]
    fn a_stream_error_does_not_overwrite_awaiting_permission() {
        // On Android a denied RECORD_AUDIO surfaces as an I/O-shaped stream
        // error; "grant the permission" is the more useful thing to keep
        // saying than a raw device message.
        let (tx, rx) = bounded::<String>(4);
        tx.try_send("permission denied".into()).unwrap();
        let mut app = app_with(MicStatus::AwaitingPermission, rx);
        app.update();
        assert_eq!(
            *app.world().resource::<MicStatus>(),
            MicStatus::AwaitingPermission
        );
    }

    #[test]
    fn a_burst_of_errors_is_drained_so_it_only_reports_once() {
        // A dying device reports repeatedly. Anything left queued would
        // re-trigger on later frames and churn the reason string.
        let (tx, rx) = bounded::<String>(4);
        for _ in 0..3 {
            tx.try_send("device disconnected".into()).unwrap();
        }
        let mut app = app_with(
            MicStatus::Connected {
                device_name: "Test Device".into(),
            },
            rx,
        );
        app.update();
        assert!(tx.is_empty(), "every queued error should have been drained");
    }
}
