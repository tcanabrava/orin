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
2. **Lessons Unit 4 "jazz"** engine prerequisites are done; what's left is
   content, and it isn't part of finishing 0.4 (`ROADMAP.md`).

No open Song Editor items remain in `TODO.md` — undo/redo, the
metronome/count-in, note audition, save/validation feedback, and the
swing/triplet grid snap are all done (see Shipped above).

3. **Repo-wide comment-shortening pass** (in progress): tightening
   overly long `///`/`//!` doc comments throughout `src/`, keeping every
   load-bearing fact (invariants, workaround reasons, cross-references)
   but cutting restatement/padding/historical narration. Done so far:
   `song_editor/{lesson_form,record,state,timeline}.rs`,
   `gameplay/{song_progress_overlay,notes,clock,adaptive_difficulty,
   state}.rs` (9 files, 5 commits).
   Comment-line counts by directory at the start of this pass (for
   picking up where it left off): `song_editor/` 1923 (29 files left),
   `gameplay/` 1627 (24 files left), `menu/`+`dialogs/`+`jam/` 1150 (39
   files), `audio_system/`+`song/`+`lessons/`+`music_score/`+top-level
   1600 (32 files). Parallel subagents repeatedly hit the session's
   usage limit mid-run with most work lost (uncommitted edits don't
   survive a killed agent) — doing this file-by-file directly instead.

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
