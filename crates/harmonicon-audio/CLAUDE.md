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
    resource (`Connected`/`Failed`); Options has a device picker and retry,
    persisted as `AudioSettings::input_device`. Startup capture is ordered
    `.after` settings load so the saved device preference wins.

- **Detection range is chart-driven:** the `PitchRange` resource (defined in
  `pitch_detect.rs`, default 200–2500 Hz) is derived from
  `Harmonica::frequency_range()` at song start and from the selected key in
  the bend trainer; both reset it on state exit. The NMF dictionary's
  staleness check includes the range — keep it that way if you add inputs.
