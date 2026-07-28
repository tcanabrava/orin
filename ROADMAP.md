# Roadmap

Where Harmonicon goes from `0.1.0`. The theme of the project is **teaching
blues and jazz harmonica through play** — every milestone should move a
self-taught player further than a YouTube tutorial would.

Near-term bug/cleanup work lives in `TODO.md`; the execution order and
implementation notes live in `PLAN.md`.

**0.2 "Trustworthy" and 0.3 "Practice" are fully shipped** (see `PLAN.md`'s
Shipped section). No release tags have been cut for either yet
(`Cargo.toml` still says `0.1.0`; the newest tag is `v0.0.1.1`); cut one
per phase once its exit criteria pass.

## 0.4 — "Blues school" (curriculum & jam) — in progress

Adaptive difficulty, the jam-session position/scale overlays, the lessons
engine with its first content pass, generated backing tracks, selectable
jam progressions and playing positions (1st/2nd/3rd, picking the matching
cross-harp key), freeform (unscored) call-and-response practice in Jam
Session (`jam::call_response`), and lessons content wave 2 (harmonica
basics, bar-counting drills, train-rhythm chugging, and the
blues-vocabulary Unit 3 — licks via call-and-response, chord-tone/
minor-blues/phrase-discipline improvisation) are all done — see
`docs/lessons_plan.md`. Open:

- **Backing track variety, remainder**: a set of recorded loops per style
  (shuffle, slow blues, swing) as a richer alternative/addition to the
  generated bass.

## 0.5 — "Content" (authoring & ecosystem)

- **Song editor maturity.** The happy-path authoring round-trip (record →
  edit → validate → play, without touching JSON) is functionally
  complete — see `PLAN.md`'s Shipped section for what's there (Record/
  Edit/Play modes, MIDI import with key suggestion, a real multi-point
  tempo map, lesson authoring alongside plain songs). What's left is
  workflow/UX maturity, found on a harmonica-player/audio/UX-focused pass
  (2026-07-27; undo/redo and the metronome/count-in are since done — see
  `PLAN.md`): no way to audition a note's pitch on click, a
  manual-placement grid that can't represent swing/triplet timing, and
  save/validation feedback that's `println!`-only (invisible outside a
  terminal). See `TODO.md`'s
  Song Editor section for the full list and `CLAUDE.md` for the detail
  behind each.
- Downloadable song packs / community sharing for the `~/Harmonicon`
  external-source folder. Live auto-refresh of that folder (songs, themes,
  and lessons) is done — see `PLAN.md`. The actual packaging/download/
  hosting mechanism for community song packs is still open — a product
  decision (where packs are hosted, how they're verified) rather than a
  small code task.
- More bundled public-domain songs across all four difficulties (see
  `TODO.md`'s content-gap item).
- Per-technique playback effects (pitch-bend/vibrato/wah DSP driven by chart
  modifiers) — a `fx_mapping`-style chart field was removed as an unbuilt
  stub; reintroduce something like it if/when this gets built.

## 0.6 — "Jazz" (advanced curriculum)

A distinct milestone from 0.4's blues curriculum — bigger scope, split out
because it needs its own content and, likely, chromatic-harmonica-specific
teaching. The lesson-level breakdown (swing-feel drills, ii–V–I arpeggio
lessons, chromatic slide curriculum, jazz-blues form) is in
`docs/lessons_plan.md`'s "Wave 2" section, Unit 4. Its engine prerequisites
(jazz chord-tone tables — `ii_v_i_chords`, `ChordQuality::{Major7,
HalfDiminished7,Dominant7Alt}` — and `Progression::JazzBlues`) are done; what
remains:

- Position work beyond blues 2nd position; chromatic slide technique as a
  first-class taught skill (the `Modifier::Slide` scoring already exists —
  see `CLAUDE.md`).
- Jazz-standard chart content — same rights/judgment-sensitive gap as
  blues content (`TODO.md`), likely worse (jazz standards are more often
  still in copyright); may need to lean on public-domain jazz-blues heads
  and original content rather than standards.

## 0.7+ — "Reach" (platforms & instruments)

- A 3D harmonica prop model for chromatic charts — `Play3D` lane geometry
  already adapts to a chromatic chart's hole count, but no matching mesh
  exists yet, so the bundled diatonic model still renders.
- Packaged releases as first-class CI artifacts: Flathub submission, Windows
  installer (workflow exists), macOS bundle (workflow exists — `release.
  yaml` builds/DMGs both architectures at tag time, `macos.yaml` checks the
  same bundling on every push/PR so a regression doesn't wait for a tag).
- Explore web build (Bevy → wasm; mic via Web Audio) for zero-install trial.
- Accessibility: mirrored layout for left-handed players, fully
  keyboard-navigable menus. (Colorblind-safe note palettes are done — an
  Options-page toggle swaps the Play2D/Play3D note highway's blow/draw
  colors for a fixed blue/yellow pair; themes can also set their own
  `colors.notes` block. The Song Editor, harmonica overlay legend, and
  song-progress/scrollbar minimaps still use their own hardcoded blue/
  orange, which already reads reasonably under red-green colorblindness —
  extend them to the same toggle if that turns out not to be enough.)
- Localization beyond en-US/pt-BR/es-ES (infrastructure is already enforced).

## Non-goals (for now)

- Multiplayer / online leaderboards.
- Non-harmonica instruments.
- Mobile (mic latency + Bevy mobile maturity make this a poor fit today).
