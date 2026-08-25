# harmonicon-gameplay

Scored play: the audio-anchored clock, note judging, the 2D/3D highways,
HUD overlays and the Bending Trainer.

Owns the clock/bar vocabulary and the overlay spawners that
`harmonicon-jam` and `harmonicon-editor` both build on, which is why it
sits below them. It must never reach *up* into either — anything they
both need lives here or lower.

Project-wide rules (workspace layering, localization, testing style,
commit conventions) are in the root `CLAUDE.md` — this file is only what's
load-bearing about *this* crate.

## Architecture (load-bearing facts)

- **Time authority:** `GameplayClock`, ticked by `tick_clock` — both in
  `gameplay/clock.rs`, along with `should_anchor_to_sink` and
  `handle_loop_boundary` (the anchoring invariant lives in that one file).
  Negative during the 3 s countdown; music starts at clock 0. Once music
  plays (and outside Jam Session) the clock is anchored to the `AudioSink`
  position via `GameplayClock::advance`, which rate-slews toward it (±0.5%
  speed, not a fixed per-frame step — a proportional nudge is
  inaudible/invisible and doesn't bias every judged offset by a constant
  amount) and snaps outright past 0.5 s of drift (a stall/seek, not
  ordinary jitter). Free-runs on frame deltas during countdown, pause, and
  Jam Session — **and always under wasm** (`SINK_POSITION_IS_RELIABLE`).
  Anchoring assumes the sink position is effectively a *hardware* clock,
  advanced by the audio device's own callback no matter how busy the game
  is; that's true natively and false in a browser, where cpal's WebAudio
  backend pulls samples from rodio inside **main-thread** callbacks
  (`setTimeout` to prime it, then each `AudioBufferSourceNode`'s `ended`
  event) and schedules them ahead onto the WebAudio timeline. A Bevy frame
  loop saturates that same thread, so the counter slips behind wall-clock
  while the audio itself plays on — and once it slips past the 0.5 s
  threshold, `advance_clock` reads it as a stall/seek and snaps the clock
  *backwards* to meet it, so the song visibly rewinds ~half a second, over
  and over. Anchoring defends against slow drift; it is not worth a
  repeated backwards jump. Two invariants, the second enforced by the type
  rather than just documented:
  - Every clock-reading system must be ordered `.after(GameplayLogic)` or
    notes stutter (see the SystemSet docs in `gameplay/plugin.rs`).
  - **Anything that jumps the clock must also seek the music sink or suspend
    anchoring** — otherwise the anchor drags the clock forward again every
    frame (the sink is always "ahead", so the correction always saturates).
    `GameplayClock`'s inner value is private; the only ways to change it are
    `set_free(t)` (anchoring guaranteed inactive — setup/countdown, Jam
    Session, Bending Trainer), `advance(dt, audio_pos)` (`tick_clock`'s own
    per-frame update), and `rewind_to(t, sink)` (jumps *and* seeks the sink
    in one call — what `handle_loop_boundary` uses, and what any future A–B
    looping UI or practice-speed feature must use too).

- **Scoring:** pure functions in `harmonicon-core`'s `scoring` (reachable
  as `harmonicon_core::scoring`, shared by
  gameplay and the song editor's practice mode), driven by the
  `score_notes` system in `gameplay/judge.rs` (alongside
  `update_active_targets`, `technique_confirmed`, `style_bonus_points`, and
  `modifier_fx_key`). `ScheduledNote`/`SongNotes` and the chart-time pure
  helpers (`target_pitch`, `resolve_item_time`, `last_note_end`,
  `LOOKAHEAD`) live in `gameplay/notes.rs`; the score/combo/config
  resources (`Score`, `SongStats`, `PitchGate`, `ScoringConfig`, …) live in
  `gameplay/state.rs`; the score HUD (`update_score_display`) lives in
  `gameplay/hud.rs`; song-lifetime setup/teardown (`reset_score`,
  `setup_scoring_config`, `detect_song_end`, `cleanup_gameplay`) lives in
  `gameplay/lifecycle.rs`. `gameplay/mod.rs` itself is wiring + re-exports
  only — every path below still resolves as `harmonicon_gameplay::gameplay::X`. Key
  concepts:
  - **Pitch identity is a MIDI note number (`u8`), not a formatted name
    string** — `PitchInfo::midi`, `ValidHarpNotes(HashSet<u8>)`,
    `PitchGate`'s `consumed: HashSet<u8>`, `ScheduledNote::expected_pitch:
    Option<u8>` (`None` for a hole/direction the harp can't produce —
    `target_pitch` returns that). This is what lets `score_notes` compare
    detected-vs-expected pitch by integer equality with zero per-frame
    allocation, and rules out enharmonic mismatches (`"A#4"` vs `"Bb4"`)
    entirely — they're the same `u8`. `note`/`octave` strings still exist on
    `PitchInfo` and `Harmonica::wind_direction_label`/`slide_label` purely
    for display; `Harmonica::wind_direction_midi` is the identity-comparison
    sibling of `wind_direction_label`. Pitch-*class* sets (`blues_scale_
    classes`, chord tones — no octave, so no MIDI number to key on) are
    still strings; that's a deliberately separate, narrower concern.
  - **Score state lives in `SongNotes` (`Vec<ScheduledNote>` + a `cursor`),
    not on ECS components.** `ScheduledNote` is plain data — this is what
    lets `gameplay_2d`/`gameplay_3d` spawn note *visuals* (`NoteVisual`/
    `NoteVisual3D`, carrying only a `note_id` index into `SongNotes::notes`)
    in a rolling `LOOKAHEAD` window instead of the whole song at once
    (`spawn_visible_notes`/`spawn_visible_notes_3d`, sharing the windowing
    logic via `notes_needing_spawn`), and lets `handle_loop_boundary` reset a
    note's state with a binary search + slice mutation instead of an ECS
    query. `notes` is kept sorted by `time` (sorted once at song load) so
    both the scoring cursor and the render window can use `partition_point`/
    early-break instead of scanning the whole song every frame. Recolor-on-
    hit systems (`update_note_visuals*`) have no `Changed<ScheduledNote>`
    filter (not a component, nothing to filter on) and just re-sync every
    currently-*spawned* note each frame — cheap, since only the window's
    worth of notes ever have a visual. Despawn-on-scroll-past needs no
    looping special case either: a note can freely despawn and get
    respawned fresh, since its score state was never on the entity to lose.
  - Candidates are scored in `|offset|`-sorted order (two-pass over
    `SongNotes`) so overlapping same-pitch notes resolve deterministically;
    notes beyond the good window are skipped before the sort (and, being
    sorted by time, end the scan outright rather than just being skipped).
  - `input_latency_ms` shifts the judged clock; calibration screen exists,
    and the results screen offers one-click application of the measured
    mean offset.
  - Bends are validated at onset via `target_pitch` (expected pitch is the
    bent one, rounded to the nearest semitone); vibrato/wah are verified
    from `(time, value)` samples collected during the sustain — measured
    oscillation rate must match the chart's `oscillation_hz` within ±40%
    (`oscillation_matches_rate`).
  - **Chord/octave-split notes** (a chart `TrackItem` with more than one
    `events` entry — `PlayMode::Chord`/`Split`) still spawn one
    `ScheduledNote` per event, but every sibling note carries the full
    target set in `ScheduledNote::chord_pitches` (empty for an ordinary
    single-event item). `score_notes` ANDs `scoring::chord_is_sounding`
    (every pitch in the set present at once) into that note's existing
    per-pitch `PitchGate` freshness check, so a chord only scores when its
    siblings are struck together — playing the same holes one at a time
    doesn't satisfy it (also excluded from `clean_attack`: a chord note is
    supposed to have company). No chart schema change was needed —
    multi-event `TrackItem`s already existed for the visual chord/split
    badge; nothing previously required their events to sound together.
    Unlike `clean_attack`, this needed no dedicated `SongStats` bucket —
    `chord_is_sounding` gates `Hit` itself, so an out-of-sync chord already
    reads as a plain miss in ordinary accuracy.
  - There's a headless end-to-end test driving `score_notes` with a
    scripted pitch stream (`end_to_end_synthetic_song_drives_score_combo_
    and_stats`) — extend it when changing scoring behaviour.

- **Adaptive difficulty** (`gameplay/adaptive_difficulty.rs`): a chart is
  divided into "sections" via the existing `TrackItem::phrase` tag (no
  schema change) — the same boundary rule `phrase_overlay` uses. Each
  section has a persisted, independent "learned" fraction
  (`profile::SongRecord::phrase_learned`, indexed by the section's ordinal
  position in the track); only a prefix of a section's notes are
  spawned/scored at a time, growing on a clean clear. Whether the feature
  is on at all is a single **global** setting
  (`settings::AdaptiveDifficultyEnabled`, an Options-menu toggle, off by
  default) — not per-song; only the learned progress itself is per-song.
  The pause menu's manual override and its own on/off toggle both take
  effect **immediately, mid-song** — `gameplay_2d`/`gameplay_3d`'s
  `resync_notes_on_adaptive_change` rebuilds `SongNotes` the moment
  `AdaptiveDifficulty` changes, carrying over already-resolved hit/miss
  state via `carry_over_note_state` (matched by `(time, hole, is_blow)`, not
  array position) so notes already judged don't reset just because the list
  was rebuilt around them; the pause-menu toggle flips
  `AdaptiveDifficultyEnabled` (persisted) and the live `AdaptiveDifficulty::
  enabled` (session cache) together, so the change is both immediate and
  becomes the new default for the next song.

- **Score HUD is message-driven, not polled:** `score_notes` emits a
  `NoteScored`-style message with the hit quality/points/new combo at the
  instant a note is judged; `update_score_display` is a `MessageReader`
  consumer, not a per-frame `format!` into `Text`. Follow this pattern for
  any future HUD element whose trigger is a discrete scoring event rather
  than a continuously-varying value.

- **The song-progress bar is a per-hole note-lanes strip, with the phrase
  overlay painted over it, and its timescale survives a music-less song**
  (`gameplay::song_progress_overlay`, shared by Play 2D/3D and Jam
  Session). The strip below the waveform spans the harmonica's whole hole
  range as `hole_count` equal lanes — the highest hole at the *top*, the
  lowest (hole 1) at the *bottom* (`note_lane_geometry`; the opposite
  vertical order from the Song Editor's own scrollbar minimap, which the
  rest of this design otherwise mirrors) — with one rectangle per note in
  its own hole's lane: left/width from `note_marker_geometry` (proportional
  to the note's own duration, floored so a very short note doesn't
  vanish), tinted blue (blow) or orange (draw), the same "note as a
  proportional colored rect" language `song_editor::interaction::
  scrollbar_marker` established for the Song Editor's scrollbar minimap —
  replacing what used to be a fixed-width white sliver with no duration,
  hole, or direction information at all. The per-phrase adaptive-
  difficulty rectangles are painted as a translucent *overlay* on that
  same strip (spawned first, so the note markers stay legible on top),
  not their own separate row below it as before — one load-bearing
  consequence: a loop-range drag can now only start in the waveform band,
  since a phrase rect covering part of the note-lanes strip intercepts
  clicks there (see `spawn_song_progress`'s own comment on the trade-off).
  Note markers are `NoteMarker{time, duration, hole, is_blow}` — a small
  type decoupled from any richer one a caller has on hand, since callers
  differ: 2D/3D map it straight from `ScheduledNote`, but Jam Session has
  no `SongNotes` at all (nothing is scored there) and instead flattens the
  chart's own `TrackItem`s to one marker per *event* (matching the scored
  modes' own per-event granularity, so a chord/split item's notes each get
  their own marker in their own lane). Separately, `spawn_song_progress`'s
  `duration_secs` is normally `SongManifest::music_duration_secs`, but a
  chart with no backing track (`SongManifest::music: None`) has nothing
  decoded to measure there — `0.0` — even though the chart itself still
  has a real length; `effective_duration` falls back to the furthest
  extent of the passed-in note markers/phrase sections in that case, so
  the bar (playhead sweep, phrase overlay, note lanes) still lays out
  against something real instead of reading as empty. Only the waveform
  row itself stays blank in that case — there's genuinely no waveform
  data without decoded audio.

- **Call-and-response** (`gameplay::call_response`): a chart's consecutive
  `TrackItem::call: true` items are one phrase. Their notes are ordinary
  `ScheduledNote`s — scored the normal way — except each carries
  `force_wait: true`, which `tick_clock`'s freeze condition
  (`wait_freeze_index`) treats like `WaitForNoteMode` being on regardless
  of the player's own toggle, so the response always waits for them. At
  song setup those same notes are also synthesized (via `song_editor::
  playback`'s synth — `PhraseNote`/`render_pcm`/`encode_wav`, widened to
  `pub(crate)` for this) into a one-shot "call" demo, scheduled to finish
  playing a fixed buffer before the phrase's first note. That playback is
  a plain fire-and-forget `AudioPlayer` spawn, like a hit-feedback sound —
  it never touches `GameplayClock` or the sink, so it can't run afoul of
  the sink-anchoring invariant above; reusing the wait-freeze path (rather
  than inventing a clock-jump) is what keeps the whole feature anchoring-
  safe and self-pacing (a slow response just delays every later cue with
  it, since the clock can't reach them before it reaches the frozen note).
