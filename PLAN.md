# Plan

Execution order and implementation notes for what's currently in flight.
Companion to `TODO.md` (the open checklist) and `ROADMAP.md` (the
destination). Once a phase ships, its detail belongs to git history, not
this file — prune it back to a one-line summary under "Shipped" below.

## Shipped

- **0.2 "Trustworthy"** — audio-synced clock, chart-derived detection
  range, mic device picker/retry, per-song persistence.
- **0.3 "Practice"** — A–B looping, practice speed, wait-for-note, tab
  display, shuffle metronome, bend trainer progression.
- **0.4, most of it** — adaptive difficulty, jam position/scale overlays,
  the full lessons engine plus a first content pass (Units 1–2),
  generated 12-bar backing ("Generate Jam"), selectable jam progressions
  (standard / quick-change / minor / jazz-blues) and playing positions
  (1st/2nd/3rd — `song::harmonica::Position::harp_key` picks the matching
  cross-harp key), and freeform (unscored) call-and-response practice in
  Jam Session (`jam::call_response` — an opt-in toggle plays a generated
  chord-tone lick, then a turn-taking banner + hole-map ghost highlight
  cue the player's echo; no scoring, distinct from the chart-scripted
  call-and-response *lesson* primitive in `gameplay::call_response`).
  Architectural invariants live in `CLAUDE.md`; the curriculum design
  lives in `docs/lessons_plan.md`.
- **Lessons content wave 2** — Unit 1 basics extensions (breathing,
  charted bends, vibrato, articulation), Unit 2 bar-counting and
  train-rhythm drills, and a new Unit 3 blues-vocabulary unit (licks via
  call-and-response, then chord-tone/minor-blues/phrase-discipline
  improvisation) — 19 lessons total, plus the three engine items wave 2
  needed: `PassCriteria::ChordToneAdherence`/`PhraseDiscipline`
  (`jam::improv::in_rest_window`/`ImprovStats::rest_violations`), and the
  lesson manifest's `progression` field. See `docs/lessons_plan.md`.
- **Physical-design restructuring** (`docs/physical_design_plan.md`) —
  all 6 phases done: layering inversions fixed (`app.rs`, `audio_system::
  synth`), inline test blobs evicted to sibling `tests.rs` files,
  `gameplay/mod.rs` and `menu/mod.rs` split into their target layouts,
  the jam feature gathered into `src/jam/`, `src/lessons.rs` split into
  `lessons/{manifest,catalog,progress}.rs`, and `tests/physical_design.rs`
  now enforces the file-size budget going forward.
- **Song editor: from bare note-grid to full authoring tool** — Record/
  Edit/Play modes with live-mic recording (onset/release debounced,
  latency-compensated, punch-in over overlapping notes), MIDI import with
  auto key-suggestion, multi-note selection, Ctrl+C/V copy-paste,
  Select/Erase/Remove/Tempo timeline tools, a real multi-point tempo map
  with an aligned waveform header, selectable out-of-scale coloring
  (`song::chart::Scale`), lesson authoring (`ContentKind::Song`/`Lesson`)
  alongside plain songs, and a UI pass (scrollable form, fixed chrome,
  per-mode button groups, a status bar). A dedicated pass since then
  (2026-07-27, playing the role of a harmonica player + audio/UX
  developer) found the remaining gaps in this workflow — see `TODO.md`'s
  Song Editor section (no swing-aware grid snap) and `CLAUDE.md`'s "Song
  editor: known gaps" bullet for the detail behind it. (Multi-selection
  drag-to-transpose — moving a lick to a different hole/chord — was
  double-checked and already works; an initial pass wrongly flagged it as
  missing.)
- **Song editor: Ctrl+Z/Ctrl+Y undo/redo** — `song_editor::undo`,
  snapshot-based (diffs `notes`/`tempo_changes` each frame `EditorState`
  changes against the last-seen snapshot, rather than instrumenting every
  mutation call site), capped at 100 entries, skipped entirely while a
  recording take is active so the whole take undoes as one step instead of
  one entry per frame of note growth. Dimmed (not disabled) Undo/Redo
  buttons alongside the keyboard shortcut. See `CLAUDE.md`'s "Ctrl+Z/
  Ctrl+Y undo and redo" bullet.
- **Song editor: metronome click + Record count-in** —
  `song_editor::metronome`, reusing `gameplay::metronome_overlay`'s pure
  tick math and shared `MetronomeTempo`/`MetronomeFeel`/`MetronomeMuted`/
  `MetronomeSounds` globals (so mute/feel preferences carry over from
  gameplay) via a new shared `play_click_if_due` extracted from that
  module's own click system, but driven by the editor's own `Playhead`
  clock instead of `GameplayClock`. Clicks during Record/Play/Practice;
  starting a fresh Record take counts in one bar first
  (`CountIn`/`begin_count_in`/`tick_count_in`/`finish_count_in`) before
  `record::start_record` actually runs, with a status-bar countdown and a
  dimmable mute-toggle button next to Undo/Redo. See `CLAUDE.md`'s "The
  Song Editor has a click track and a Record count-in" bullet.
- **Song editor: audition a note's pitch on selection** —
  `song_editor::audition`, a short (0.6s) blip of a note's resolved pitch
  the instant `EditorState::selected_note` changes to a different note,
  reusing `audio_system::synth`'s additive harmonica voice
  (`playback::note_freq`/`render_pcm`) rather than a separate
  reference-tone generator, so it matches what the note actually sounds
  like in Play/Practice/Record. Every existing selection call site (a
  fresh placement, clicking an existing note, Ctrl+click, paste) already
  funnels through the same `selected_note`, so none needed touching. See
  `CLAUDE.md`'s "The Song Editor auditions a note's pitch on selection"
  bullet.
- **Song editor: Save/Load feedback in the status bar, not just the log**
  — `song_editor::save_feedback`. `harpchart`'s and `lesson_form`'s
  Save/Load systems used to report every outcome with a bare `println!`;
  each now also sets a localized success/warning/failure `SaveFeedback`
  message, shown by `panel::update_status_bar` as its own top-priority
  tier for 4s before falling back to whatever it'd otherwise show, while
  still logging via `info!`/`warn!` for a developer at a terminal.
  `lesson_form::serialize_lesson` now returns its validation warnings
  (empty id/unit, schema failure) instead of printing them, so a save can
  report "saved with warnings" instead of a plain "Saved" — except the
  locale-key-pairs-to-add reminder, deliberately left console-only (a
  multi-line copy-paste block, not a one-line status). See `CLAUDE.md`'s
  "Save/Load outcomes show up in the status bar, not just the log"
  bullet.
- **Code-duplication cleanup** (whole-tree duplicate-block scan,
  2026-07-19) — all 6 phases done, no behavior changes, `cargo test`/
  `cargo clippy`/`tests/physical_design.rs` clean throughout:
  `gameplay::notes::build_scheduled_notes` (+ `play_mode_label`) replaces
  `gameplay_2d`/`gameplay_3d`'s own near-identical note builders, and
  `adaptive_difficulty::rebuild_song_notes` replaces their duplicated
  `resync_notes_on_adaptive_change` middles; `gameplay_2d::{harp_pitches,
  step_hole_glow}` is now the shared per-cell glow step
  `update_holes`/`update_holes_3d` both call, and
  `gameplay_2d::spawn_blow_draw_legend` is the shared blow/draw legend
  (`note_tint`/`update_note_visuals`'s 2D/3D pairs were deliberately left
  separate — genuinely different render targets, no real savings from
  unifying); `gameplay::harmonica_overlay::spawn_diagram` is one
  parameterized grid builder behind all three harmonica-diagram spawners;
  `song_editor::playback` gained shared `secs_per_tick`/`playhead_for`/
  `spawn_background_music` used by `playback.rs`/`practice.rs`/
  `record.rs`, `song_editor::state::overlapping_group` replaced the
  duplicated transitive-overlap walk in `enforce_direction`/`enforce_expr`,
  and `meta_form::spawn_cycle_row`/`panel_widgets::spawn_button_shell`
  unified the click-to-cycle and plain-button scaffolds respectively; MIDI
  parsing (`tick_to_seconds`/`collect_tempo_map`/`track_name_of`/
  `note_on_count`/`extract_notes`) is now `song::midi`, a new public
  module `song_editor::midi_import` builds on; small menu/UI fry
  (`results::spawn_stat_row`, `options`'s slider-row scaffold,
  `jam_generate`'s stepper rows, `menu::pages::lessons`'s reader-line
  spawn) each collapsed to one shared local helper; and the literal
  duplicate `richter_harp` reference-layout tests in
  `bending_trainer/tests.rs` were deleted in favor of `song::harmonica::
  tests`'s copies.
- **Build-time message-registration check** — `build.rs` now statically
  scans for every `#[derive(Message)]` type and fails the build if it's
  never registered with `.add_message::<T>()` anywhere, the same class of
  bug that shipped once (`ExternalFolderChanged`, fixed in
  `assets_management/mod.rs`) and only surfaced as a runtime panic the
  first time its `MessageReader`/`MessageWriter` system actually ran. Same
  static/textual approach as the existing localization-literal scan; see
  `CLAUDE.md`'s "Message registration is enforced" bullet.
- **Packaging CI fixes** — `flatpak.yaml`'s build was failing on a fresh
  runner (but not locally) because `flatpak-builder` shells out to the
  host's `eu-strip` (from `elfutils`) after the build to split/strip
  debuginfo, and `apt-get install --no-install-recommends` was skipping it
  (only a `Recommends` of the `flatpak-builder` package, not a hard
  dependency) — fixed by installing `elfutils` explicitly. Separately,
  macOS packaging gained the same "catch it on every push, not just at a
  tag" treatment `flatpak.yaml` already gave Linux: a new
  `.github/workflows/macos.yaml` builds the release binary, assembles the
  same bare `.app` bundle `release.yaml`'s tag-triggered
  `release-macOS-intel`/`release-macOS-apple-silicon` jobs already produce,
  and `hdiutil`-packages it into a `.dmg` (native-arch only — Apple
  Silicon, since this is a packaging-regression check, not a release
  artifact), uploading the result as a short-retention build artifact.
- **0.5: live auto-refresh of the external song/theme/lesson folders** —
  `assets_management::watch` starts one recursive `notify-debouncer-full`
  watcher (same crate Bevy's own `file_watcher` feature uses internally,
  added as our own direct, always-on dependency rather than flipping that
  Bevy feature — see `CLAUDE.md`'s Asset sources bullet for why) on
  `~/Harmonicon` at Startup, if it exists, and fires one generic
  `ExternalFolderChanged{top_level_dirs}` message per debounced batch
  (`watch::changed_top_level_dirs`, pure/unit-tested) — deliberately
  agnostic of what `songs`/`themes`/`lessons` mean, since this module is
  low-level shared vocabulary and those are feature concerns above it.
  `assets_management::mod.rs` consumes it for `songs`/`themes`
  (`rescan_on_external_change`); `lessons::catalog` consumes the same
  message for `lessons` from the other side
  (`rescan_lessons_on_external_change`) — a `lessons`-depends-on-
  `assets_management` edge, not the reverse. A successful live rescan
  fires its own specific `SongsRescanned`/`ThemesRescanned`/
  `LessonsRescanned` message; the Artist List, Theme picker, and Lessons
  list pages each consume theirs to force a same-page rebuild if that page
  happens to be open when a drop-in happens. No manual refresh button, no
  restart. The rest of that roadmap item — actual downloadable/community
  song packs — is still open; see `ROADMAP.md`'s 0.5 section.
- **Options: fullscreen toggle** — `settings::FullscreenEnabled`
  (persisted, off by default) plus `settings::apply_fullscreen` mirroring
  it onto the primary window's `WindowMode` (borderless, not exclusive
  fullscreen); a pill-button toggle on the Options page, same shape as the
  adaptive-difficulty toggle.
- **Song-progress bar: per-hole note lanes, phrase overlay, survives no
  background music** — the note strip below the waveform spans the
  harmonica's full hole range as equal lanes, highest hole at the top,
  lowest at the bottom, with each note a rectangle in its own hole's
  lane, sized to its own duration and tinted blue (blow)/orange (draw) —
  the same "note as a colored proportional rect" language the Song
  Editor's scrollbar minimap already used. See `CLAUDE.md`'s
  song-progress-bar bullet.
- **Menu pages auto-scroll instead of silently overflowing** — the Artist
  List (and any other page whose content can outgrow the screen) used to
  just grow past the top/bottom edges with no way to reach the rest.
  `menu::scene::spawn_menu_root` now spawns a generic
  `dialogs::scroll_area::spawn_scroll_area` widget and returns its entity
  instead of the outer root's, so all 22 existing call sites gained
  scrolling with zero changes. See `CLAUDE.md`'s menu-scrolling bullet.
- **0.6: jazz engine prerequisites** — `song::harmonica::ii_v_i_chords`
  (a standalone ii–V–I chord-tone builder), three new `ChordQuality`
  variants (`Major7`/`HalfDiminished7`/`Dominant7Alt`), and
  `Progression::JazzBlues` (a VI7–ii7–V7 turnaround inside the 12-bar
  form) — both plug into existing generic machinery (`jam::session`'s
  chord-tone classification, `jam::backing`'s bass generation, the
  Progression-generic twelve-bar overlay) with no further changes needed.
  What's left for Lessons Unit 4 is content only — see `TODO.md`.
- **Alternate harmonica tunings** — `song::harmonica::
  paddy_richter_harp`/`natural_minor_harp`, matching the existing
  Richter/Country shape (`BendingProfile` variant + reference-layout
  builder); bend availability falls out of the existing blow/draw-gap math
  for free, no per-tuning bend logic needed.
- **Accessibility: colorblind-safe note palette** — an Options-page toggle
  (`settings::ColorblindPalette`) swaps the Play2D/Play3D note highway's
  blow/draw colors for a fixed blue/yellow pair; themes can also set their
  own `colors.notes` block. Scope: only the scored note highway honors the
  toggle today — see `ROADMAP.md`'s 0.7+ section for what's still using
  hardcoded colors.
- **`phrase_learned` stable keying** — adaptive-difficulty progress is now
  keyed by a phrase section's own name (disambiguated for repeats) instead
  of its ordinal position in the track, so re-editing a chart's phrase
  tags no longer silently misapplies old progress to the wrong section.
- **Song editor: swing/triplet-aware grid snap** — `TICKS_PER_BEAT` went
  4 → 12 (the lowest resolution divisible by both 4, for straight 16ths,
  and 3, for triplets), so a genuine triplet tick position exists on the
  grid at all — the old 4-tick grid had no integer position for one. A
  chart's own `timing.resolution` is self-describing and every tick/time
  conversion already takes it as an explicit parameter rather than
  hardcoding the constant, so existing bundled charts (still at
  `resolution: 4`) needed no migration. A new Straight/Shuffle/Triplet
  toggle (`song_editor::state::SnapMode`, next to the harmonica-kind
  toggle in the meta form) constrains where a click on the grid lands: 16
  straight ticks (0,3,6,9 — reproducing the old any-of-4-ticks grid at the
  new resolution), a 2:1 long-short swing pair (0,8), or three equal
  triplet subdivisions (0,4,8). The grid's sub-beat lines are now tiered
  by color (`theme::SongEditorColors::triplet_line`, new) rather than one
  line per raw tick, which at resolution 12 would be 11 lines per
  beat — cluttered well past readable.
- **Jam Session: MIDI multi-track backing with per-track mute** — a song
  can ship `song/music.mid` instead of `music.ogg`/`.wav`; `song::loader`
  renders each non-empty track to its own `AudioSource` at load time
  (`song::midi::render_track_pcm`, the shared additive harmonica-voice
  synth) rather than one pre-mixed file. Jam Session plays every track as
  its own synchronized sink (all spawned the same frame) and shows a
  horizontal mute-toggle row below the 12-bar/harmonica columns
  (`jam::midi_tracks`) — muting a track is just zeroing that sink's
  volume, no live re-mixing. See `CLAUDE.md`'s "A song can ship a raw MIDI
  file as its backing track" bullet for the full design.

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
