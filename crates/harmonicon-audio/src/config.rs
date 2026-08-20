// SPDX-License-Identifier: MIT

//! The live audio configuration resource.
//!
//! Lives here rather than in `crate::settings` — which persists it — so that
//! `audio_system` needn't depend on `settings` to read the device and
//! algorithm it captures with. `settings` depends on `audio_system`, never
//! the reverse (`docs/physical_design_plan.md` rule 2).

use bevy::prelude::*;

use super::pitch_detect::PitchAlgorithm;

/// Player-tunable audio levels (0.0–1.0, linear), read by the audio spawners
/// (song music, metronome clicks) and edited on the Options page. Persisted
/// by `crate::settings`; adjusting the music level updates the playing song
/// in real time.
#[derive(Resource)]
pub struct AudioSettings {
    pub music_volume: f32,
    pub metronome_volume: f32,
    /// Milliseconds subtracted from the gameplay clock when judging whether
    /// a detected pitch was played in time. Compensates for the microphone
    /// input pipeline (FFT window ≈ 46 ms, OS buffer, cpal callback).
    /// Typical values: 50–100 ms for USB/built-in microphones.
    pub input_latency_ms: i32,
    /// Which algorithm the audio pipeline uses to detect played pitches.
    pub pitch_algorithm: PitchAlgorithm,
    /// Preferred microphone input device name; empty means "use the system
    /// default". Read by `audio_system::audio_input::start_capture`, which
    /// falls back to the default if this device isn't currently plugged in.
    pub input_device: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.8,
            metronome_volume: 0.7,
            input_latency_ms: 0,
            pitch_algorithm: PitchAlgorithm::default(),
            input_device: String::new(),
        }
    }
}
