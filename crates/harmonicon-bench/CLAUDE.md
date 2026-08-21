# harmonicon-bench

Pitch-detection benchmarking and the synthetic dataset generator.
Developer tooling, not shipped game code.

Project-wide rules (workspace layering, localization, testing style,
commit conventions) are in the root `CLAUDE.md` — this file is only what's
load-bearing about *this* crate.

## Architecture (load-bearing facts)

- **Pitch-detection quality work is benchmark-first**, per `Harmonicon Note
  Detection Roadmap.md` (repo root, not checked in — a personal working doc):
  don't change the detection algorithms themselves until there's a
  reproducible dataset of recordings to measure against. `song_editor::
  debug_record` (`--features dev`) is the recording side — its "Debug
  Recording" checkbox dumps a take's raw mic audio + the chart + metadata
  (algorithm, FFT/hop/window, a live detected-notes log) into
  `assets/debug_songs/<song name>/`. `note_bench` (`crates/harmonicon-bench/src/note_bench.rs`'s
  pure, unit-tested comparison logic, driven by the `src/bin/note_bench.rs`
  binary — `cargo run --bin note_bench`) is the analysis side: replays every
  recording found there through all five algorithms and prints a hit/miss/
  phantom summary plus the most common confusion pairs. `compare`'s
  `false_positive` count includes an *extra* detected pitch sitting alongside
  an otherwise-correct one (not just a detection where nothing was expected
  at all) — e.g. NMF's dictionary matching reporting neighboring semitones
  as "also active" for a single clean note is exactly the kind of recurring
  phantom the roadmap's "Analyze the Errors" step is after.
