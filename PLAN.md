# Plan

Execution order and implementation notes for what's currently in flight.
Companion to `TODO.md` (the open checklist) and `ROADMAP.md` (the
destination). Once a phase ships, its detail belongs to git history, not
this file — prune it back to a one-line summary under "Shipped" below.

## Shipped

Full design detail for anything below lives in `CLAUDE.md` (architecture)
and git history (implementation); this list is intentionally just a
one-line index of what's landed.

- **0.2 "Trustworthy"** — audio-synced clock, chart-derived detection
  range, mic device picker/retry, per-song persistence.
- **0.3 "Practice"** — A–B looping, practice speed, wait-for-note, tab
  display, shuffle metronome, bend trainer progression.
- **0.4, most of it** — adaptive difficulty, jam position/scale overlays,
  the lessons engine + content wave 1, generated 12-bar backing,
  selectable jam progressions/positions, freeform call-and-response in
  Jam Session.
- **Lessons content wave 2** — Units 1–3 (basics extensions, bar-counting/
  train-rhythm drills, blues-vocabulary licks + improvisation), 19 lessons.
- **Lessons content wave 3** — new Unit 4 "Scales and Improvisation" (6
  lessons: major/minor-pentatonic/country scale run drills, major/minor-
  pentatonic open-jam improvisation, quick-change improvisation) plus the
  `JamScale` engine work making Jam Session's live scale feedback
  configurable instead of hardcoded to blues (`docs/lessons_plan.md`).
- **Lessons Unit 5 "jazz", 4 of 5 lessons** — swing-eighths, ii-V-I chord
  tones, jazz-blues form (open jam), chromatic slide basics; all pure
  content, no engine work needed. "Jazz heads" (actual repertoire) is
  deliberately not built — blocked on specific pieces confirmed public
  domain, not just content authoring (`docs/lessons_plan.md`).
- **Genre-selectable Jam Session backing** — a `Genre` axis (Blues/Jazz/
  Rock/Reggae/Country, `jam::backing`) alongside the existing `Progression`,
  each with its own bass rhythm pattern and straight/swing feel; a new
  "Genre" combobox on Generate Jam. Also fixed that page's comboboxes
  getting their open dropdowns clipped (a `ScrollArea` clips to its
  content's own height, not the full window, when content is shorter than
  available space) via a new non-scrolling `spawn_menu_root_plain`.
- **Harmonica rhythm guide + metronome swing-visual fix** — a live pulse
  row in Jam Session (`jam::rhythm_guide`) showing when the picked genre's
  groove wants a harmonica attack, reusing `jam::backing::genre_pattern`'s
  rhythm data directly so it can't drift from the bass audio it's modeled
  on. Along the way, fixed the metronome's visual beat dot to actually
  pulse twice per beat in Shuffle feel (matching the audio click, which
  already did) instead of a single beat-long decay.
- **Circle-of-fifths lesson** — a new `dialogs::circle_of_fifths` diagram
  widget (this crate's first circular UI layout) visualizing that a
  harmonica `Position` is just a step count around the circle of fifths;
  `Position` also grew `Fourth`/`Fifth`/`Twelfth` variants.
- **Circle-of-fifths, live in Jam Session** — `jam::position_guide` adds a
  live position compass (always shown in Jam Session) and a
  `position_cycle` lesson mechanic that calls a new position every 4 bars,
  reusing the existing `ScaleAdherence` scoring against a moving target
  instead of a fixed one; new lesson `circle-of-fifths-jam`.
- **Acyclic module graph, enforced** — broke every dependency cycle
  (`settings ↔ audio_system`, and the `{gameplay, jam, menu, song_editor}`
  component covering two thirds of the tree) and added
  `no_module_dependency_cycles` to `tests/physical_design.rs` to keep it
  that way, with no allowlist. Prerequisite for the workspace split, since
  Cargo cannot express a circular crate dependency.
- **Physical-design restructuring** — layering fixes, inline tests evicted
  to `tests.rs` files, `gameplay`/`menu`/`lessons` split into their target
  layouts, `jam` gathered into `src/jam/`, a file-size budget test.
- **Song editor: full authoring tool** — Record/Edit/Play modes with
  live-mic recording, MIDI import, multi-select, copy-paste,
  Select/Erase/Remove/Tempo tools, a real tempo map, selectable
  out-of-scale coloring, lesson authoring alongside plain songs.
- **Song editor: undo/redo, metronome + count-in, pitch audition on
  select, save/validation feedback in the status bar, swing/triplet grid
  snap** — the UX pass that closed out the authoring tool's remaining gaps.
- **Code-duplication cleanup** (whole-tree scan) — shared note-builder/
  glow-step/legend/diagram/MIDI-parsing helpers replacing near-identical
  copies across gameplay 2D/3D, the Song Editor, and menus.
- **Build-time message-registration check** — `build.rs` fails the build
  if a `#[derive(Message)]` type is never registered, instead of only
  surfacing as a runtime panic.
- **Packaging CI fixes** — Flatpak `eu-strip` dependency fix; macOS
  packaging now checked on every push, not just at a release tag.
- **0.5: live auto-refresh of `~/Harmonicon`** — songs/themes/lessons
  dropped into the external folder show up without a manual refresh or
  restart.
- **Options: fullscreen toggle**.
- **Song-progress bar: per-hole note lanes + phrase overlay**, survives a
  song with no backing track.
- **Menu pages auto-scroll** instead of silently overflowing.
- **0.6: jazz engine prerequisites** — ii–V–I chord tables, jazz-blues
  progression. Content authoring is what's left (`TODO.md`).
- **Alternate harmonica tunings** — Paddy Richter, natural minor.
- **Accessibility: colorblind-safe note palette** (Play 2D/3D highway).
- **`phrase_learned` stable keying** — adaptive-difficulty progress keyed
  by phrase name, not track position.
- **Jam Session: MIDI multi-track backing with per-track mute**.
- **Shared music-notation staff** (`src/music_score/`, Bravura/SMuFL) —
  below the song-progress bar in Play 2D/3D and in the Song Editor.
- **Compact layout for narrow windows** (`src/responsive.rs`) — Play
  2D/3D and the Song Editor adapt below a shared width breakpoint; menus
  were already scroll-safe and out of scope.
- **Android/iOS prep, desktop-verifiable groundwork** — on-screen
  equivalents for every keyboard-only action found (UI zoom, the Song
  Editor's Delete/Copy/Paste, the spectrogram's style cycle), plus
  `MicStatus::AwaitingPermission` groundwork for a future mobile
  permission-prompt flow. The actual Android/iOS build config
  (`#[bevy_main]`, `[package.metadata.android]`, an Xcode project) is
  deliberately deferred — this sandbox has no NDK and can never have
  Xcode, so writing unverifiable config wasn't worth it yet. See
  `ROADMAP.md`'s Mobile section (currently still listed as a non-goal —
  worth revisiting given this) for the full research findings (cpal
  already has real Android/iOS mic input support; Bevy's own mobile
  toolchain story is real but immature).

## Current work

Finishing 0.4:

1. **Backing track variety, remainder** (0.4): recorded loops per style
   (shuffle, slow blues, swing) as a richer alternative to the generated
   bass — real audio content, not a code task.
2. **Lessons Unit 5 "jazz"** — 4 of 5 lessons shipped (see Shipped above);
   what's left is just "jazz heads", blocked on rights-verified repertoire,
   and it isn't part of finishing 0.4 (`ROADMAP.md`).

No open Song Editor items remain in `TODO.md` — undo/redo, the
metronome/count-in, note audition, save/validation feedback, and the
swing/triplet grid snap are all done (see Shipped above).

**Note detection benchmarking** (see `Harmonica Note Detection Roadmap.md`,
repo root, not checked in): a **harmonica constraint solver**
(`song::harmonica_constraints::plausible_notes` — rejects any simultaneous
blow+draw pitch mix, keeps chords/octaves) and a **synthetic benchmark
dataset generator** (`synthetic_dataset.rs` / `cargo run --bin
gen_synthetic_dataset`, writing into `assets/debug_songs/` in
`note_bench`'s own format, as a stand-in until real recordings exist) are
done — `note_bench` prints a `<algorithm>+HC` row showing the solver's
effect. It reliably cuts NMF's phantom count (e.g. single notes 67→43,
octaves 26→15 in the synthetic benchmark) but occasionally drops a genuine
hit too (its majority-wind-direction heuristic misjudging).

**Deliberately not wired into live gameplay yet** — decided, not just
not-gotten-to:
- Only validated against synthetic (sample-accurate, noiseless) audio; a
  real reed's blow/draw transition frames could get needlessly stripped
  the same way a phantom does.
- A dropped note costs the player score/combo live, while an un-filtered
  phantom is already mostly harmless to scoring (it just doesn't match
  the expected pitch) — so this trade needs real debug recordings to
  confirm it's a net win before it's worth building, since right now
  there's no plumbing at all to thread the active chart's `Harmonica`
  down to where `PitchEvent` is consumed (`audio_system` deliberately
  doesn't depend on `song`).
- When it does go in, scope it to NMF only — FFT/YIN/MPM barely
  benefited in the synthetic benchmark.
- Blocked on: a real harmonica + real debug recordings via
  `song_editor::debug_record` (`--features dev`), then re-running
  `note_bench` against them before revisiting this decision.

3. **Repo-wide comment-shortening pass — done.** Every directory under
   `src/` (`song_editor/`, `gameplay/`, `menu/`, `dialogs/`, `jam/`,
   `audio_system/`, `song/`, `lessons/`, `music_score/`,
   `assets_management/`, `bin/`, `spectrogram/`, and the top-level
   `src/*.rs` files) has been swept file by file for overly long
   `///`/`//!` doc comments, tightening restatement/padding and cutting
   historical narration ("used to be", "the old code") while keeping
   every load-bearing fact (invariants, workaround reasons, cross-
   references). A final full-repo scan turned up nothing left worth
   trimming — remaining dense blocks are genuinely load-bearing technical
   rationale (SMuFL glyph geometry in `music_score/`, psychoacoustic bass
   in `jam/backing.rs`, etc.), not padding. Parallel subagents repeatedly
   hit the session's usage limit mid-run with most work lost (uncommitted
   edits don't survive a killed agent, tried twice); doing this file-by-
   file directly worked reliably. Comment-only edits were committed
   without a build/test/clippy cycle per file, per explicit instruction —
   a final sanity build was still run at the end of the whole pass.

## Working practices

- Keep the pure-logic/ECS split: new mechanics get pure functions + unit
  tests first, systems second.
- Update `docs/gameplay_validation.md` whenever a phase adds a mode or
  changes timing behaviour.
- Chart schema changes must stay backward compatible (new fields optional);
  bump `metadata.format_version` when adding any.
- One phase per release; cut a tag when the phase's exit criteria pass —
  none have been cut yet even though 0.2/0.3 are done (see `ROADMAP.md`).
- Prune this file as work lands — a "done" item belongs in git history and,
  if it's an architectural invariant future code must respect, in
  `CLAUDE.md`; it doesn't need to live here too.
