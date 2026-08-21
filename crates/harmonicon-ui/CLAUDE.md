# harmonicon-ui

Reusable presentation with no gameplay knowledge: the widget library,
the SMuFL notation staff and the audio visualiser.

A widget here must work for *any* caller. Anything that knows what a
note, a score or a lesson is belongs in the feature that owns it.

Project-wide rules (workspace layering, localization, testing style,
commit conventions) are in the root `CLAUDE.md` — this file is only what's
load-bearing about *this* crate.

## Architecture (load-bearing facts)

- **A shared music-notation staff renders with the Bravura SMuFL font**
  (`harmonicon-ui`'s `music_score/`, a new top-level module, sibling to `spectrogram` —
  used by Play 2D/3D, below the song-progress bar, and by the Song Editor,
  in its own fixed chrome below the grid). Deliberately coarse, not a
  sight-reading tool: noteheads (whole/half/filled by duration), stems,
  ledger lines, sharp accidentals, ties across a bar line, and a single
  eighth-note flag — no beaming, no sixteenth-or-shorter flag tier, no
  dotted durations, single treble clef only. `assets/fonts/Bravura.otf`
  (SIL OFL, `Bravura-OFL.txt` alongside it) is bundled and loaded the same
  `Font::from_bytes` way as `dialogs::font_fallback`'s small icon fonts;
  every glyph codepoint and every relative measurement (notehead width,
  stem attachment point, ledger-line extension) comes straight from
  Bravura's own published `bravura_metadata.json` — the one thing that
  *couldn't* be derived that way, `GLYPH_BASELINE_CORRECTION` (correcting
  for Bevy positioning a `Text` node's bounding box top-left rather than
  the font's own SMuFL-relative glyph origin), is an estimate, flagged in
  its own doc comment as the first thing to adjust by eye.
  - **`NotationNote { start_beat, duration_beats, midi }`** (beats, not
    ticks or seconds) is the module's only input — it never touches a
    chart's tempo map or an editor's own tick resolution, so each of the
    three call sites converts its own time representation first: gameplay
    (`gameplay::music_score_bridge`) goes `ScheduledNote::time` (seconds)
    through `song::chart::seconds_to_tick`; the Song Editor
    (`song_editor::music_score_bridge`) just divides its own
    already-tempo-independent `GridNote::tick` by `TICKS_PER_BEAT`. This
    is the same split `gameplay::metronome_overlay`/`song_editor::
    metronome` already use for the same reason (two genuinely different
    clocks/note models) — not duplicated logic, since the actual
    rendering stays 100% inside `music_score` either way.
  - **A note that crosses one or more bar lines is split into per-bar
    segments and tied together** (`split_at_bar_lines`, called by both
    bridges with their own `beats_per_bar`) rather than drawn as one
    oversized notehead — `NotationNote::tied_from_previous` marks every
    segment after the first, which suppresses that segment's own
    accidental (a tie doesn't restate one) and draws a tie mark back to
    the segment before it. The tie itself is a *real curved arc*, not a
    flat rectangle: `music_score::tie_material::TieMaterial` is a
    `UiMaterial` fragment shader (`assets/shaders/music_score_tie.wgsl`),
    the same "custom shader for a shape a plain `Node` can't express"
    pattern `gameplay::note_tail_2d::NoteTail2dMaterial` already
    established — one shared material handle covers every tie, since
    unlike the note tail this shape never varies. Its bounding box spans
    the *real* pixel gap between the two tied noteheads' own onset
    positions (computed from the immediately-preceding entry in
    `MusicScoreNotes`, which `split_at_bar_lines` guarantees is the
    tied-from segment), not a fixed size — an earlier version centered a
    fixed-width box on the second note alone and never actually reached
    the first.
  - **The visible window sizes itself from the panel's own on-screen
    width**, independent of any other host UI (the falling-note highway's
    own lookahead, the Song Editor grid's own column count): `visible_
    beats(panel_width_px)` derives how many beats fit on each side of the
    "now" reference line from `ComputedNode` (read via a `MusicScorePanel`
    marker on the panel's root), converted from physical to logical px the
    same way `gameplay_2d::size_note_tails` already has to. Rebuilds also
    fire on `Changed<ComputedNode>` (first layout pass, a window resize),
    not just on note/playhead changes. The Song Editor's own
    `MusicScorePlayhead` — which only gameplay's clock naturally drives —
    falls back to `EditorState::scroll_beat` whenever nothing is actually
    playing, so panning the grid pans the score with it instead of the
    score staying pinned whichever beat playback last stopped at (usually
    0).
  - **`dialogs::font_fallback::apply_font_fallback` runs on *every* `Text`
    entity in the game**, not just the ones its own small icon fonts
    cover — for a single-run string whose character isn't one of its own
    known gaps, it unconditionally resets `TextFont.font` back to
    `FontSource::default()`, since that system's whole contract assumes
    it's the only thing ever touching `TextFont.font`. `music_score`'s
    Bravura glyphs (Private-Use-Area SMuFL codepoints, never in that
    system's own gap lists) got silently clobbered back to the default
    font — the actual cause of an early "tofu box" bug, not a font or
    Bevy/Parley limitation. Fixed with a general opt-out marker,
    `dialogs::font_fallback::SkipFontFallback`, rather than special-casing
    SMuFL codepoints into that system.
  - **This surfaced a real, unrelated harp-model bug while testing**: the
    Song Editor's own `state::overblow_ok` allowed Overblow on holes 1–6,
    but `song::harmonica::hole_notes` only defines a real overblow reed
    for 1/4/5/6 — holes 2/3 fell through the gap (the editor accepted the
    click, but no pitch existed anywhere downstream: scoring, playback,
    or this staff). Fixed `overblow_ok` to match `hole_notes` exactly.
