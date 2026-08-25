# harmonicon-audio

Microphone capture and real-time pitch detection.

**Keep this crate free of song/scoring/UI concepts** — it sits directly
above `harmonicon-core` and below everything else.

Project-wide rules (workspace layering, localization, testing style,
commit conventions) are in the root `CLAUDE.md` — this file is only what's
load-bearing about *this* crate.

## Architecture (load-bearing facts)

- **Audio input path:** cpal callback → mono downmix → 4096-sample chunks
  with 50% overlap (`harmonicon-audio`'s `audio_input.rs`) → crossbeam channel →
  `process_audio` in `main.rs` → one FFT per chunk (`pitch_detect::analyze`)
  → `PitchEvent` message + `AudioFrame` resource (shared with spectrogram).
  Five selectable algorithms (FFT/YIN/pYIN/MPM/NMF) in
  `harmonicon-audio`'s `pitch_detect.rs`.
  - The capture callback must stay allocation-free: chunk buffers come from
    a recycling pool (`AudioCapture::free_sender`); `process_audio` returns
    the previous `AudioFrame::samples` buffer to that pool each frame. Keep
    that contract intact when touching either side.
  - Mic lifecycle lives in `start_capture(&mut World)` + the `MicStatus`
    resource (`Connected`/`Failed`/`AwaitingPermission`); Options has a
    device picker and retry,
    persisted as `AudioSettings::input_device`. Startup capture is ordered
    `.after` settings load so the saved device preference wins.
  - **`start_capture` asks permission before it opens anything.**
    `permission.rs` exposes `microphone_granted()`/`request_microphone()`
    with the same names on every target — off Android they answer "granted"
    and do nothing, so the call site needs no `#[cfg]`. On Android they call
    `Activity.checkSelfPermission`/`requestPermissions` over JNI (via the
    JavaVM/Activity pointers `android-activity` publishes through
    `ndk_context`). Until the user answers, capture parks in
    `MicStatus::AwaitingPermission` rather than `Failed`: opening a cpal
    input stream without `RECORD_AUDIO` fails in a way indistinguishable
    from a broken device, and that distinction is the whole reason the
    variant exists. `retry_capture_when_permission_granted` (an `Update`
    system, registered in the root `lib.rs`) then **polls** until it's
    granted — the result is delivered to an `onRequestPermissionsResult`
    callback on a Java activity this codebase doesn't own, and a
    once-per-install dialog doesn't justify routing that back over JNI. A
    JNI error is reported as *not* granted, deliberately: the honest answer
    when the check itself couldn't run.

- **Detection range is chart-driven:** the `PitchRange` resource (defined in
  `pitch_detect.rs`, default 200–2500 Hz) is derived from
  `Harmonica::frequency_range()` at song start and from the selected key in
  the bend trainer; both reset it on state exit. The NMF dictionary's
  staleness check includes the range — keep it that way if you add inputs.
