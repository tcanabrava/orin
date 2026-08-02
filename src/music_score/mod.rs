// SPDX-License-Identifier: MIT

//! A scrolling music-notation staff, rendered with the
//! [Bravura](https://github.com/steinbergmedia/bravura) SMuFL font — shared
//! by the Song Editor and both gameplay render modes, all three of which
//! spawn [`spawn_music_score`] wherever their own layout calls for it and
//! drive it by writing [`MusicScoreNotes`]/[`MusicScorePlayhead`].
//!
//! Deliberately not full music engraving: noteheads (whole/half/filled by
//! duration), stems, ledger lines, and sharp accidentals only — no
//! beaming, no eighth/sixteenth flags, no ties, single treble clef only.
//! This is a supplementary visual, not a sight-reading tool (the Song
//! Editor's own tab readout already exists for players who want exact
//! rhythm) — see `docs/lessons_plan.md`'s framing of the tab readout for
//! the same reasoning applied here.
//!
//! Every glyph's relative geometry (notehead width, stem attachment point,
//! ledger-line extension) is taken directly from Bravura's own published
//! `bravura_metadata.json` (values in "staff spaces," SMuFL's own unit —
//! 1 staff space is the gap between two adjacent staff lines), not
//! estimated. The one thing that *couldn't* be derived that way:
//! [`GLYPH_BASELINE_CORRECTION`] — see its own doc comment for why, and
//! flag it as the first thing to adjust by eye if glyphs don't look
//! vertically centered on their staff position.

use bevy::prelude::*;

use crate::audio_system::midi::midi_to_note;

// ── SMuFL glyphs (Bravura) ──────────────────────────────────────────────
//
// Codepoints are standardized by the SMuFL specification itself (every
// conformant SMuFL font, not just Bravura, uses the same ones) — see
// https://w3c.github.io/smufl/latest/tables/ or `glyphnames.json` in the
// https://github.com/w3c/smufl repo.

mod glyph {
    pub const G_CLEF: &str = "\u{E050}";
    pub const NOTEHEAD_WHOLE: &str = "\u{E0A2}";
    pub const NOTEHEAD_HALF: &str = "\u{E0A3}";
    pub const NOTEHEAD_BLACK: &str = "\u{E0A4}";
    pub const ACCIDENTAL_SHARP: &str = "\u{E262}";
}

// ── Pure notation logic ──────────────────────────────────────────────────

/// One note to draw on the staff. `start_beat`/`duration_beats` are in
/// quarter-note units — deliberately *not* ticks or seconds, so this
/// module never needs to know about a chart's tempo map or an editor's own
/// tick resolution; every caller converts its own time representation into
/// beats before handing notes over (see each call site: `song_editor`'s
/// ticks are already tempo-independent multiples of a beat; gameplay's
/// `ScheduledNote::time` in seconds goes through `song::chart::
/// seconds_to_tick` first). `midi` is the actual sounded pitch (bends/
/// overblow/overdraw/slide already resolved) — the same identity every
/// other pitch comparison in the codebase uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NotationNote {
    pub start_beat: f64,
    pub duration_beats: f64,
    pub midi: u8,
}

/// Diatonic staff step for `midi`, treble clef, where E4 (the staff's
/// bottom line) is step 0 and each step is one staff position (a line or a
/// space) — *not* one semitone. This is what makes a sharp share its
/// natural neighbor's step, distinguished only by an accidental glyph (see
/// [`needs_sharp`]): staff position is decided by the note's *letter
/// name*, not its exact pitch.
pub fn staff_step(midi: u8) -> i32 {
    let name = midi_to_note(midi as i32); // sharp-only spelling, e.g. "C#4", "E4"
    let bytes = name.as_bytes();
    let has_sharp = bytes.get(1) == Some(&b'#');
    let letter = bytes[0] as char;
    let octave: i32 = name[if has_sharp { 2 } else { 1 }..].parse().unwrap_or(4);
    let letter_index = match letter {
        'C' => 0,
        'D' => 1,
        'E' => 2,
        'F' => 3,
        'G' => 4,
        'A' => 5,
        'B' => 6,
        _ => 2,
    };
    (octave * 7 + letter_index) - (4 * 7 + 2) // relative to E4
}

/// Whether `midi` needs a sharp accidental drawn before its notehead. This
/// codebase spells every accidental as a sharp, never a flat (see
/// `audio_system::midi::midi_to_note`'s own doc comment), so a sharp is the
/// only accidental glyph the staff ever needs to draw.
pub fn needs_sharp(midi: u8) -> bool {
    midi_to_note(midi as i32).as_bytes().get(1) == Some(&b'#')
}

/// The staff steps (see [`staff_step`]) a ledger line is drawn at for a
/// note at `step` — empty while `step` is within the staff (`0..=8`).
/// Ledger lines fall at every *even* step from the staff's own edge out to
/// (and including, if even) `step` itself; a note sitting in the
/// intervening odd step (a space just outside the staff, e.g. D4 just
/// below the staff, or G5 just above it) needs none at all — the classic
/// "middle C gets one ledger line, the note in the space just below it
/// still reads against that same line" rule.
pub fn ledger_line_steps(step: i32) -> Vec<i32> {
    let mut out = Vec::new();
    if step < 0 {
        let mut s = -2;
        while s >= step {
            out.push(s);
            s -= 2;
        }
    } else if step > 8 {
        let mut s = 10;
        while s <= step {
            out.push(s);
            s += 2;
        }
    }
    out
}

/// Which notehead shape a note's duration gets. Deliberately coarse — see
/// the module doc comment on engraving scope: quarter notes and anything
/// shorter all get the same filled notehead + plain stem, with no flags or
/// beaming to distinguish an eighth from a quarter. Thresholds sit at the
/// midpoint between adjacent standard durations (2 and 4 beats) so a
/// slightly-off recorded/quantized duration still classifies as intended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoteheadKind {
    Whole,
    Half,
    Filled,
}

impl NoteheadKind {
    fn glyph(self) -> &'static str {
        match self {
            NoteheadKind::Whole => glyph::NOTEHEAD_WHOLE,
            NoteheadKind::Half => glyph::NOTEHEAD_HALF,
            NoteheadKind::Filled => glyph::NOTEHEAD_BLACK,
        }
    }

    /// A whole note conventionally draws no stem at all.
    fn has_stem(self) -> bool {
        self != NoteheadKind::Whole
    }

    /// Notehead width, in staff spaces — `bravura_metadata.json`'s own
    /// `glyphBBoxes` (`noteheadWhole`/`noteheadHalf`/`noteheadBlack`).
    fn width_sp(self) -> f32 {
        match self {
            NoteheadKind::Whole => 1.688,
            NoteheadKind::Half | NoteheadKind::Filled => 1.18,
        }
    }
}

pub fn notehead_kind(duration_beats: f64) -> NoteheadKind {
    if duration_beats >= 3.0 {
        NoteheadKind::Whole
    } else if duration_beats >= 1.5 {
        NoteheadKind::Half
    } else {
        NoteheadKind::Filled
    }
}

// ── Layout constants ──────────────────────────────────────────────────────

/// Total height (px) of the score panel. `pub` so callers reserve this
/// much extra space below wherever they place it — the same "the overlay
/// paints on top of its own fixed footprint, callers pad around it"
/// pattern `gameplay::song_progress_overlay::BAR_HEIGHT` already
/// established.
pub const PANEL_HEIGHT: f32 = 120.0;

/// Pixel gap between two adjacent staff lines — the base unit everything
/// else in this module scales from. 1 "staff space" (SMuFL's own unit,
/// used throughout `bravura_metadata.json`) equals this many pixels,
/// by construction (see [`GLYPH_FONT_PX`]).
const STAFF_LINE_SPACING: f32 = 9.0;
/// Pixels per staff *step* (a line or a space is half a staff-space gap).
const STEP_PX: f32 = STAFF_LINE_SPACING / 2.0;
/// Distance from the panel's own top edge to the staff's top line (F5,
/// step 8) — chosen to leave headroom above for a handful of ledger
/// lines, since a harmonica's playable range routinely sits well outside
/// a single treble-clef staff.
const STAFF_TOP_MARGIN: f32 = 50.0;
/// SMuFL's own convention: a font's em size is set so that 4 staff spaces
/// equal 1 em ("staffSpace = 0.25 em") — this is what makes a glyph drawn
/// at this font size match a staff built from [`STAFF_LINE_SPACING`].
const GLYPH_FONT_PX: f32 = STAFF_LINE_SPACING * 4.0;

/// Bevy positions a `Text` node's bounding-box top-left at the `Node`'s own
/// `top`/`left` — not the font's internal glyph baseline/origin
/// `bravura_metadata.json`'s own coordinates are expressed relative to.
/// There's no published mapping from one to the other short of measuring
/// Bravura's actual OpenType vertical metrics against Bevy's own text
/// shaper (cosmic-text) at a specific font size. This constant is the
/// correction for that gap — **it was estimated, not measured** (this
/// development environment has no display to render against and compare),
/// so it's the first thing to adjust by eye if noteheads/the clef don't
/// look vertically centered on their intended staff line/space once this
/// is actually visible on screen. Every glyph in this module applies it
/// uniformly, so recalibrating is a one-constant fix, not a hunt through
/// per-glyph offsets.
const GLYPH_BASELINE_CORRECTION: f32 = GLYPH_FONT_PX * 0.5;

/// Notehead stem attachment points, in staff spaces relative to the
/// notehead's own origin (its bounding box's bottom-left corner) —
/// `bravura_metadata.json`'s `glyphsWithAnchors.noteheadBlack`
/// (`noteheadHalf` carries the same two anchors). Only `Filled`/`Half`
/// noteheads have a stem at all (see [`NoteheadKind::has_stem`]), and
/// both share these same anchors.
const STEM_UP_ANCHOR_SP: (f32, f32) = (1.18, 0.168);
const STEM_DOWN_ANCHOR_SP: (f32, f32) = (0.0, -0.168);
/// Standard engraving stem length, in staff spaces (3.5 staff spaces is
/// the conventional default length for an ordinary stem).
const STEM_LENGTH_SP: f32 = 3.5;
const STEM_THICKNESS_SP: f32 = 0.12; // engravingDefaults.stemThickness

/// How far a ledger line extends beyond the notehead on each side, and how
/// thick it is — both `bravura_metadata.json`'s own `engravingDefaults`
/// (`legerLineExtension`/`legerLineThickness`).
const LEDGER_EXTENSION_SP: f32 = 0.4;
const LEDGER_THICKNESS_SP: f32 = 0.16;

/// `accidentalSharp`'s own bounding-box width (`glyphBBoxes.accidentalSharp`
/// — `bBoxNE.x - bBoxSW.x` = `0.996 - 0.0`), plus a small fixed gap before
/// the notehead it belongs to.
const ACCIDENTAL_SHARP_WIDTH_SP: f32 = 0.996;
const ACCIDENTAL_GAP_SP: f32 = 0.2;

/// The staff step the gClef glyph's own SMuFL origin is anchored on — the
/// G4 line, i.e. the second line from the bottom of a treble-clef staff
/// (this is *why* it's called a G clef: the glyph's curl circles that
/// line). Not the same as [`staff_step`]'s numbering by coincidence — it's
/// the same "E4 = 0, one integer per staff position" scheme throughout.
const GCLEF_ANCHOR_STEP: i32 = 2;

const CLEF_X: f32 = 8.0;
/// Where the "now" reference line sits, and where a note at the current
/// playhead position draws — notes scroll right to left through it, the
/// same "things move toward a fixed reference line" language the falling-
/// note highway and song-progress playhead already use elsewhere.
const PLAYHEAD_X: f32 = 56.0;
const PIXELS_PER_BEAT: f32 = 34.0;
/// How far ahead of the playhead (in beats) a note is still spawned.
const VISIBLE_BEATS_AHEAD: f64 = 12.0;
/// How far *behind* the playhead a note stays spawned before being culled
/// — a small trailing margin so a just-played long note doesn't vanish
/// the instant its onset crosses the reference line, only once it's
/// fully done sounding.
const VISIBLE_BEATS_BEHIND: f64 = 1.0;

fn y_for_step(step: i32) -> f32 {
    STAFF_TOP_MARGIN + (8 - step) as f32 * STEP_PX
}

// ── Resources ──────────────────────────────────────────────────────────────

/// The loaded Bravura font handle, `pub` so a caller's own setup system
/// can pass it into [`spawn_music_score`] (which needs it immediately, to
/// set the clef glyph's `TextFont`, rather than waiting a frame for
/// [`rebuild_score_notes`] to discover it).
#[derive(Resource, Clone)]
pub struct BravuraFont(pub Handle<Font>);

/// The chart's notes to draw, already converted to beat-based
/// [`NotationNote`]s by whichever caller populated this — see the module
/// doc comment for why the conversion happens at each call site instead of
/// here. Typically built once, at song/session setup, not every frame.
#[derive(Resource, Default)]
pub struct MusicScoreNotes(pub Vec<NotationNote>);

/// The current "now" position, in the same beat units as
/// [`MusicScoreNotes`] — updated every frame by whichever caller is
/// currently active (the Song Editor's own playhead, or the gameplay
/// clock converted through the chart's tempo map).
#[derive(Resource, Default)]
pub struct MusicScorePlayhead(pub f64);

/// The (persistent) child of the panel that [`rebuild_score_notes`]
/// despawns and respawns the note glyphs under — everything *except* this
/// layer (the staff lines, the clef, the reference line) is spawned once
/// by [`spawn_music_score`] and never touched again.
#[derive(Component)]
struct MusicScoreNotesLayer;

/// Tags every entity [`rebuild_score_notes`] spawns, so the next rebuild
/// knows what to despawn first.
#[derive(Component)]
struct MusicScoreNoteGlyph;

// ── Plugin ───────────────────────────────────────────────────────────────

pub struct MusicScorePlugin;

impl Plugin for MusicScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicScoreNotes>()
            .init_resource::<MusicScorePlayhead>()
            .add_systems(Startup, load_bravura_font)
            .add_systems(
                Update,
                rebuild_score_notes.run_if(
                    resource_changed::<MusicScoreNotes>
                        .or_else(resource_changed::<MusicScorePlayhead>),
                ),
            );
    }
}

fn load_bravura_font(mut fonts: ResMut<Assets<Font>>, mut commands: Commands) {
    const BYTES: &[u8] = include_bytes!("../../assets/fonts/Bravura.otf");
    commands.insert_resource(BravuraFont(fonts.add(Font::from_bytes(BYTES.to_vec()))));
}

// ── Spawning ───────────────────────────────────────────────────────────────

/// Spawns the persistent part of the score panel — background, the 5
/// staff lines, the clef, the "now" reference line, and an empty notes
/// layer [`rebuild_score_notes`] fills in every time [`MusicScoreNotes`]/
/// [`MusicScorePlayhead`] change. The panel carries no assumption about
/// where it sits on screen beyond its own fixed [`PANEL_HEIGHT`] — each
/// caller places it in their own layout (below `song_progress_overlay`'s
/// bar in gameplay; wherever fits in the Song Editor's own chrome).
pub fn spawn_music_score(parent: &mut ChildSpawnerCommands, bravura: &BravuraFont) -> Entity {
    let mut root = parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(PANEL_HEIGHT),
            overflow: Overflow::clip_x(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
    ));
    let root_id = root.id();
    root.with_children(|panel| {
        // The 5 staff lines: steps 0, 2, 4, 6, 8.
        for line in 0..5 {
            let step = line * 2;
            panel.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(y_for_step(step)),
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.75, 0.75, 0.80, 0.6)),
            ));
        }
        // Clef — its own SMuFL origin sits on the G4 line (GCLEF_ANCHOR_STEP).
        panel.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CLEF_X),
                top: Val::Px(y_for_step(GCLEF_ANCHOR_STEP) - GLYPH_BASELINE_CORRECTION),
                ..default()
            },
            Text::new(glyph::G_CLEF),
            TextFont {
                font: FontSource::Handle(bravura.0.clone()),
                font_size: FontSize::Px(GLYPH_FONT_PX),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        // "Now" reference line — notes scroll toward/through this the same
        // way the falling-note highway approaches its own hit line.
        panel.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PLAYHEAD_X),
                top: Val::Px(0.0),
                width: Val::Px(1.0),
                height: Val::Px(PANEL_HEIGHT),
                ..default()
            },
            BackgroundColor(Color::srgba(0.95, 0.80, 0.35, 0.5)),
        ));
        // Notes layer: positioned so its own local x=0 IS the playhead —
        // every note glyph spawned inside it is placed at
        // `(note.start_beat - now) * PIXELS_PER_BEAT`, which can (and
        // routinely does) go negative for a note just behind the
        // playhead; `overflow: clip_x()` on the panel itself keeps that
        // from spilling past the panel's own left edge.
        panel.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PLAYHEAD_X),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            MusicScoreNotesLayer,
        ));
    });
    root_id
}

fn rebuild_score_notes(
    mut commands: Commands,
    bravura: Option<Res<BravuraFont>>,
    notes: Res<MusicScoreNotes>,
    playhead: Res<MusicScorePlayhead>,
    layers: Query<Entity, With<MusicScoreNotesLayer>>,
    existing: Query<Entity, With<MusicScoreNoteGlyph>>,
) {
    let Some(bravura) = bravura else { return };
    for glyph in &existing {
        commands.entity(glyph).despawn();
    }

    let now = playhead.0;
    for layer in &layers {
        commands.entity(layer).with_children(|parent| {
            for note in &notes.0 {
                if note.start_beat + note.duration_beats < now - VISIBLE_BEATS_BEHIND
                    || note.start_beat > now + VISIBLE_BEATS_AHEAD
                {
                    continue;
                }
                spawn_note_glyphs(parent, &bravura, note, now);
            }
        });
    }
}

fn spawn_note_glyphs(
    parent: &mut ChildSpawnerCommands,
    bravura: &BravuraFont,
    note: &NotationNote,
    now: f64,
) {
    let x = ((note.start_beat - now) * PIXELS_PER_BEAT as f64) as f32;
    let step = staff_step(note.midi);
    let kind = notehead_kind(note.duration_beats);
    let notehead_y = y_for_step(step);

    if needs_sharp(note.midi) {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(
                    x - (ACCIDENTAL_SHARP_WIDTH_SP + ACCIDENTAL_GAP_SP) * STAFF_LINE_SPACING,
                ),
                top: Val::Px(notehead_y - GLYPH_BASELINE_CORRECTION),
                ..default()
            },
            Text::new(glyph::ACCIDENTAL_SHARP),
            TextFont {
                font: FontSource::Handle(bravura.0.clone()),
                font_size: FontSize::Px(GLYPH_FONT_PX),
                ..default()
            },
            TextColor(Color::WHITE),
            MusicScoreNoteGlyph,
        ));
    }

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(notehead_y - GLYPH_BASELINE_CORRECTION),
            ..default()
        },
        Text::new(kind.glyph()),
        TextFont {
            font: FontSource::Handle(bravura.0.clone()),
            font_size: FontSize::Px(GLYPH_FONT_PX),
            ..default()
        },
        TextColor(Color::WHITE),
        MusicScoreNoteGlyph,
    ));

    if kind.has_stem() {
        let stem_up = step < 4; // below the middle line (B4) -> stem up
        let (anchor_x_sp, anchor_y_sp) = if stem_up {
            STEM_UP_ANCHOR_SP
        } else {
            STEM_DOWN_ANCHOR_SP
        };
        let stem_x = x + anchor_x_sp * STAFF_LINE_SPACING;
        let stem_notehead_y = notehead_y - anchor_y_sp * STAFF_LINE_SPACING;
        let stem_len_px = STEM_LENGTH_SP * STAFF_LINE_SPACING;
        let stem_top = if stem_up {
            stem_notehead_y - stem_len_px
        } else {
            stem_notehead_y
        };
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(stem_x - STEM_THICKNESS_SP * STAFF_LINE_SPACING * 0.5),
                top: Val::Px(stem_top),
                width: Val::Px(STEM_THICKNESS_SP * STAFF_LINE_SPACING),
                height: Val::Px(stem_len_px),
                ..default()
            },
            BackgroundColor(Color::WHITE),
            MusicScoreNoteGlyph,
        ));
    }

    let ledger_width_px = (kind.width_sp() + 2.0 * LEDGER_EXTENSION_SP) * STAFF_LINE_SPACING;
    let ledger_left = x - LEDGER_EXTENSION_SP * STAFF_LINE_SPACING;
    for ledger_step in ledger_line_steps(step) {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ledger_left),
                top: Val::Px(y_for_step(ledger_step)),
                width: Val::Px(ledger_width_px),
                height: Val::Px(LEDGER_THICKNESS_SP * STAFF_LINE_SPACING),
                ..default()
            },
            BackgroundColor(Color::srgba(0.75, 0.75, 0.80, 0.6)),
            MusicScoreNoteGlyph,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staff_step_places_e4_at_the_bottom_line() {
        assert_eq!(staff_step(64), 0); // E4 = MIDI 64
    }

    #[test]
    fn staff_step_places_middle_c_two_steps_below_the_staff() {
        assert_eq!(staff_step(60), -2); // C4 = MIDI 60
    }

    #[test]
    fn staff_step_places_f5_at_the_top_line() {
        assert_eq!(staff_step(77), 8); // F5 = MIDI 77
    }

    #[test]
    fn staff_step_shares_the_same_step_as_its_sharp() {
        // C4 and C#4 sit on the same staff position; only the accidental differs.
        assert_eq!(staff_step(60), staff_step(61));
    }

    #[test]
    fn needs_sharp_is_true_only_for_a_sharp_spelling() {
        assert!(!needs_sharp(60)); // C4
        assert!(needs_sharp(61)); // C#4
        assert!(!needs_sharp(64)); // E4
    }

    #[test]
    fn ledger_line_steps_is_empty_within_the_staff() {
        assert!(ledger_line_steps(0).is_empty());
        assert!(ledger_line_steps(4).is_empty());
        assert!(ledger_line_steps(8).is_empty());
    }

    #[test]
    fn ledger_line_steps_middle_c_gets_exactly_one() {
        assert_eq!(ledger_line_steps(-2), vec![-2]);
    }

    #[test]
    fn ledger_line_steps_the_space_just_outside_the_staff_needs_none() {
        // D4 (step -1, the space directly below the bottom line) and G5
        // (step 9, the space directly above the top line) both sit closer
        // to the staff than any ledger line would be.
        assert!(ledger_line_steps(-1).is_empty());
        assert!(ledger_line_steps(9).is_empty());
    }

    #[test]
    fn ledger_line_steps_a_note_in_the_gap_beyond_the_first_ledger_still_gets_it() {
        // B3 (step -3, a space) is one step further than middle C (-2) —
        // it still reads against that same first ledger line.
        assert_eq!(ledger_line_steps(-3), vec![-2]);
        // A3 (step -4, on a line) needs two.
        assert_eq!(ledger_line_steps(-4), vec![-2, -4]);
    }

    #[test]
    fn ledger_line_steps_above_the_staff_mirrors_below() {
        assert_eq!(ledger_line_steps(10), vec![10]); // A5
        assert_eq!(ledger_line_steps(11), vec![10]); // B5, a space
        assert_eq!(ledger_line_steps(12), vec![10, 12]); // C6
    }

    #[test]
    fn notehead_kind_classifies_by_duration() {
        assert_eq!(notehead_kind(4.0), NoteheadKind::Whole);
        assert_eq!(notehead_kind(3.0), NoteheadKind::Whole);
        assert_eq!(notehead_kind(2.0), NoteheadKind::Half);
        assert_eq!(notehead_kind(1.5), NoteheadKind::Half);
        assert_eq!(notehead_kind(1.0), NoteheadKind::Filled);
        assert_eq!(notehead_kind(0.25), NoteheadKind::Filled);
    }

    #[test]
    fn whole_notes_have_no_stem() {
        assert!(!NoteheadKind::Whole.has_stem());
        assert!(NoteheadKind::Half.has_stem());
        assert!(NoteheadKind::Filled.has_stem());
    }
}
