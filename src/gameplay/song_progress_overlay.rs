// SPDX-License-Identifier: MIT

//! A song-progress bar pinned to the top of the screen, shared by the 2D and
//! 3D gameplay views. It shows the song's whole waveform (pre-analyzed at
//! asset-load time — see `audio_system::waveform`/`SongManifest::waveform`)
//! so a player picking a loop range can see where in the song they're
//! aiming, with a thin red playhead line (styled like the Song Editor's
//! `PlayheadLine`) marking the current position.
//!
//! Below the waveform, a note-lanes strip spans the harmonica's full hole
//! range — highest hole at top, lowest at bottom (opposite the Song
//! Editor's own scrollbar minimap, which the "note as a proportional rect"
//! language is otherwise modeled on: `song_editor::interaction::
//! scrollbar_marker`). Each note is a rectangle in its hole's lane, sized to
//! its real duration and tinted by blow/draw. The per-phrase
//! adaptive-difficulty rectangles (dim-gray to green by learned fraction)
//! paint as a translucent overlay on that same strip — spawned before the
//! note markers so the notes stay legible on top.
//!
//! A song with no background music (`SongManifest::music: None`) has no
//! waveform and a `music_duration_secs` of `0.0`, but the chart itself
//! still has real length — [`spawn_song_progress`] falls back to the
//! notes'/phrase-sections' own extent as the bar's timescale instead of
//! reading as empty (see its own doc comment). Only the waveform row stays
//! empty in that case.
//!
//! The bar has two [`ProgressBarMode`]s: pure visualization while playing,
//! or — while paused — editable, click-and-drag anywhere to sweep out a new
//! A–B loop range (live yellow preview). Releasing fires
//! [`RequestLoopRange`] rather than writing `LoopConfig` directly, keeping
//! the drag interaction and the loop-adoption policy decoupled.

use bevy::picking::events::{Click, Drag, DragEnd, DragStart, Pointer};
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_render::prelude::MaterialNode;

use crate::app::AppState;

use super::adaptive_difficulty::{AdaptiveDifficulty, PhraseSection};
use super::pause_menu::SelectedPhraseIndex;
use super::song_waveform_material::{
    SongWaveformMaterial, SongWaveformMaterialPlugin, pack_amplitudes,
};
use super::{GameplayClock, GameplayRoot, LoopConfig, Paused, SongEnd, loop_range_valid};

/// Every bar keeps at least this much height (as a fraction 0..1) even during
/// silence, so the waveform reads as a continuous shape rather than gaps.
const WAVEFORM_FLOOR: f32 = 0.04;

/// Height (px) of the waveform section.
const WAVEFORM_HEIGHT: f32 = 26.0;

/// Waveform fill color — a desaturated, receding gray-blue, deliberately
/// *not* the same hue as a note marker's blow color one row down. The two
/// used to share almost the same light blue, which made "there's sound
/// here" and "this is a blow note" read as the same signal in one panel.
const WAVEFORM_COLOR: Color = Color::srgba(0.55, 0.62, 0.72, 0.55);

/// Height (px) of the note-lanes strip below the waveform — tall enough
/// that even a 12-hole chromatic's lanes stay individually legible (5px
/// each; a 10-hole diatonic gets 6px). See [`note_lane_geometry`].
const NOTE_LANES_HEIGHT: f32 = 60.0;

/// Border width for a phrase-section rectangle (semi-transparent fill, fully
/// opaque border — see [`phrase_fill_color`]) so adjacent sections stay
/// visually distinct even when their fill colors are close.
const PHRASE_RECT_BORDER: f32 = 1.5;

/// Total height (px) of the bar, pinned at the very top of the screen.
/// `pub` so gameplay HUDs can reserve this much space instead of placing
/// content under it, where the bar (painted above them — see
/// [`BAR_Z_INDEX`]) would cover it.
pub const BAR_HEIGHT: f32 = WAVEFORM_HEIGHT + HUD_DIVIDER_HEIGHT + NOTE_LANES_HEIGHT;

/// Narrowest a note's marker is ever drawn (fraction 0..1 of the bar),
/// regardless of how short the note actually is — a proportionally-accurate
/// but sub-pixel-wide marker would be invisible. See [`note_marker_geometry`].
const MIN_NOTE_MARKER_FRAC: f32 = 0.003;

/// A freshly-spawned note-marker rect's fill, before [`sync_note_marker_colors`]
/// corrects it to the actual active theme/colorblind colors — used only so
/// a marker isn't fully invisible for the one frame between spawning and
/// that system's first run (see its own doc comment for why spawn time
/// itself can't just resolve the real color directly). Same hues the Song
/// Editor's scrollbar minimap uses by default
/// (`song_editor::interaction::SCROLLBAR_BLOW_COLOR`/`SCROLLBAR_DRAW_COLOR`),
/// for the same "note as a colored rect" language.
const NOTE_MARKER_BLOW_FALLBACK: Color = Color::srgba(0.50, 0.75, 1.00, 0.9);
const NOTE_MARKER_DRAW_FALLBACK: Color = Color::srgba(1.00, 0.62, 0.35, 0.9);

/// Alpha applied to whichever blow/draw [`crate::theme::NoteColors`] a note
/// marker resolves to (the theme colors themselves carry no alpha of their
/// own) — matches the old hardcoded markers' opacity.
const NOTE_MARKER_ALPHA: f32 = 0.9;

/// Alpha for a phrase-section's full-height fill — low, so it reads as a
/// soft background tint behind the note markers rather than a block of
/// color competing with them for attention. See [`PHRASE_ACCENT_ALPHA`]
/// for the readable "how learned is this" signal.
const PHRASE_FILL_ALPHA: f32 = 0.20;

/// Alpha for a phrase-section's accent stripe ([`PhraseAccentStripe`]) —
/// high enough that the mastery gradient stays glanceable from a slim strip
/// instead of needing the whole lane height saturated to be legible.
const PHRASE_ACCENT_ALPHA: f32 = 0.85;

/// Height (px) of a phrase-section's accent stripe, along its top edge.
const PHRASE_ACCENT_HEIGHT: f32 = 6.0;

/// Height (px) of the seam drawn between the waveform and note-lanes rows —
/// `crate::theme::HUD_DIVIDER_COLOR` reused for every seam in the song-
/// timeline HUD, including `music_score::spawn_music_score`'s own border to
/// the note-lanes row above it.
const HUD_DIVIDER_HEIGHT: f32 = 1.0;

/// Above the pause overlay's own backdrop (`pause_menu::setup_pause_menu`,
/// `GlobalZIndex(200)`) so the bar renders on top of — and, since bevy_ui's
/// picking backend hit-tests in the same order it paints, actually receives
/// clicks through — the pause dimming rather than being buried under it.
const BAR_Z_INDEX: i32 = 250;

/// The moving playhead; a thin vertical line, styled like the Song Editor's
/// `PlayheadLine`. Its horizontal position is driven each frame.
#[derive(Component, Default, Clone)]
pub struct ProgressPlayhead;

/// Highlights the current A–B loop range within the bar — either the
/// committed `LoopConfig` range, or a live preview of an in-progress drag
/// (see [`LoopDrag`]). Absolutely positioned (unlike the waveform bars,
/// which occupy normal flex flow) so it can sit at an arbitrary offset
/// independent of them.
#[derive(Component, Default, Clone)]
pub struct LoopRangeMarker;

/// One rectangle in the per-phrase adaptive-difficulty strip, spanning its
/// `PhraseSection`'s time range. `usize` is the section's ordinal index
/// (into `AdaptiveDifficulty::sections`/`learned`), used by
/// [`update_phrase_section_colors`] to re-tint it without respawning
/// whenever a phrase's learned fraction changes (e.g. a manual pause-menu
/// edit).
#[derive(Component, Clone, Copy)]
struct PhraseSectionRect(usize);

/// The compact, higher-opacity accent stripe along a phrase-section rect's
/// top edge — same section index as its parent [`PhraseSectionRect`], kept
/// as its own component (rather than reading it back off the parent) so
/// [`update_phrase_section_colors`] can re-tint it with one more disjoint
/// query instead of walking the hierarchy.
#[derive(Component, Clone, Copy)]
struct PhraseAccentStripe(usize);

/// Marks a note-marker rectangle with which direction it is, so
/// [`sync_note_marker_colors`] can tint it from the active theme/colorblind
/// setting without `spawn_song_progress` (called from `gameplay_2d`/
/// `gameplay_3d`/`jam::session`'s own already near Bevy's system-param-limit
/// setup systems) needing `LoadedTheme`/`ColorblindPalette` threaded in as
/// extra params just to resolve a color.
#[derive(Component, Clone, Copy)]
struct NoteMarkerRect {
    is_blow: bool,
}

/// The single entity that accepts click-and-drag input for setting a loop
/// range — the bar's own root. Every visual child (waveform bars, note
/// markers, the loop marker, the playhead) is spawned `Pickable::IGNORE`,
/// so a click can never get swallowed by whichever tiny rectangle happens
/// to be on top; it always lands on this one surface instead.
#[derive(Component, Default, Clone)]
struct ProgressBarDragSurface;

/// The timescale the waveform bars are laid out on — real decoded audio
/// duration, set once when the bar spawns. Deliberately not [`SongEnd`]
/// (last chart note + a fixed tail): a tightly-trimmed track ends before
/// that tail elapses, a padded one keeps going after it, and positioning
/// the playhead/loop marker against the wrong one drifts them out of sync
/// with the waveform underneath.
#[derive(Resource, Default)]
pub struct AudioDuration(pub f64);

/// Whether the progress bar behaves as a static visualization (normal play)
/// or accepts click-and-drag to set an A–B loop range (while paused). Kept
/// as its own resource — rather than every system here reaching for `Paused`
/// directly — so "is the bar editable right now" is one explicit concept,
/// synced from `Paused` by [`sync_progress_bar_mode`].
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ProgressBarMode {
    #[default]
    Visualization,
    Edit,
}

/// An in-progress click-and-drag on the bar, sweeping out a candidate A–B
/// loop range. `origin_time`/`current_time` are the drag's two endpoints in
/// song-seconds, in the order the pointer visited them (not sorted — the
/// drag can go either direction), live-updated so [`update_loop_marker`] can
/// preview it before the mouse is released.
#[derive(Resource, Default)]
struct LoopDrag {
    active: bool,
    origin_time: f64,
    current_time: f64,
}

/// Fired when a click-and-drag on the progress bar (in edit mode) is
/// released, proposing `start_time..end_time` as the new A–B loop range.
/// Consumed by [`apply_requested_loop_range`] — the only system that writes
/// `LoopConfig::start_time`/`end_time` here — so the drag interaction itself
/// only ever *requests* a range, never applies one directly.
#[derive(Message, Debug, Clone, Copy)]
pub struct RequestLoopRange {
    pub start_time: f64,
    pub end_time: f64,
}

/// One note's timing/direction/hole, as much as the note-lanes strip needs
/// — decoupled from any richer caller type, since callers differ: scored
/// modes (2D/3D) have `gameplay::notes::ScheduledNote`, but Jam Session
/// (nothing scored there) builds these directly from the chart's track
/// events instead.
#[derive(Clone, Copy)]
pub struct NoteMarker {
    pub time: f64,
    pub duration: f64,
    pub hole: u8,
    pub is_blow: bool,
}

/// Spawns the full-width progress bar at the top of the screen: the
/// waveform (from `waveform`, one entry per bar in 0..1, see
/// `SongManifest::waveform`), then the note-lanes strip (`hole_count`
/// lanes, highest hole at top) with the phrase-section overlay painted over
/// it, then the loop marker and playhead on top of everything. Tagged
/// `GameplayRoot` so it's torn down with the scene. `duration_secs` is the
/// audio's real length if there is one (see [`AudioDuration`]); a song with
/// no background music falls back to the furthest extent of
/// `notes`/`sections` instead (see [`effective_duration`]) so the bar still
/// has a real timescale — only the waveform row stays empty in that case.
pub fn spawn_song_progress(
    commands: &mut Commands,
    waveform: &[f32],
    duration_secs: f64,
    notes: &[NoteMarker],
    hole_count: u8,
    sections: &[PhraseSection],
    learned: &[f32],
) {
    let duration_secs = effective_duration(duration_secs, notes, sections);
    commands.insert_resource(AudioDuration(duration_secs));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(BAR_HEIGHT),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(crate::theme::HUD_PANEL_BG),
            GlobalZIndex(BAR_Z_INDEX),
            GameplayRoot,
            Button,
            RelativeCursorPosition::default(),
            ProgressBarDragSurface,
        ))
        .observe(on_drag_start)
        .observe(on_drag)
        .observe(on_drag_end)
        .with_children(|bar| {
            // A single shader-drawn node (`SongWaveformMaterial`/
            // `assets/shaders/song_waveform.wgsl`) instead of one `Node`
            // per bucket — the shader interpolates between buckets and
            // mirrors the fill around the vertical center, so this reads
            // as a smooth audio-waveform silhouette rather than a row of
            // isolated bar-chart spikes. The material asset itself can't
            // be created here (`spawn_song_progress` only has `Commands`,
            // not `Assets<SongWaveformMaterial>`, and every caller's own
            // setup system is already near Bevy's system-param limit) —
            // deferred via `commands.queue` instead, same "no extra system
            // param needed" trick `audio_input::start_capture` uses.
            let waveform_node = bar
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(WAVEFORM_HEIGHT),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();
            let waveform_amplitudes = pack_amplitudes(waveform, WAVEFORM_FLOOR);
            bar.commands().queue(move |world: &mut World| {
                let mut materials = world.resource_mut::<Assets<SongWaveformMaterial>>();
                let handle = materials.add(SongWaveformMaterial {
                    color: WAVEFORM_COLOR.into(),
                    amplitudes: waveform_amplitudes,
                });
                world.entity_mut(waveform_node).insert(MaterialNode(handle));
            });

            // Seam between the waveform and note-lanes rows — otherwise
            // flush with no visual boundary at all.
            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(HUD_DIVIDER_HEIGHT),
                    ..default()
                },
                BackgroundColor(crate::theme::HUD_DIVIDER_COLOR),
                Pickable::IGNORE,
            ));

            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(NOTE_LANES_HEIGHT),
                    position_type: PositionType::Relative,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|strip| {
                // Phrase-section overlay first, so the note markers below
                // paint over it and stay legible rather than getting
                // tinted by whatever learned-fraction color the section
                // underneath happens to be.
                if duration_secs > 0.0 {
                    for (i, section) in sections.iter().enumerate() {
                        let Some((left, width)) = phrase_rect_geometry(
                            section.start_time,
                            section.end_time,
                            duration_secs,
                        ) else {
                            continue;
                        };
                        let learned_frac = learned.get(i).copied().unwrap_or(0.0);
                        // Deliberately pickable (unlike everything else in
                        // this bar) — see `on_phrase_rect_click`. Its
                        // default `should_block_lower: true` means a drag
                        // starting over a phrase rect no longer reaches
                        // `ProgressBarDragSurface`, so a loop-range drag has
                        // to start in the waveform band instead — an
                        // accepted trade for making sections clickable.
                        // Note markers spawned after this are
                        // `Pickable::IGNORE` so they don't also shadow it.
                        strip
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Percent(left * 100.0),
                                    top: Val::Px(0.0),
                                    width: Val::Percent(width * 100.0),
                                    height: Val::Percent(100.0),
                                    border: UiRect::all(Val::Px(PHRASE_RECT_BORDER)),
                                    ..default()
                                },
                                // `colorblind: false` here — this is only
                                // the spawn-time value; `update_phrase_
                                // section_colors` (which does have
                                // `ColorblindPalette`) corrects it the same
                                // frame or the next.
                                BackgroundColor(phrase_fill_color(learned_frac, false)),
                                BorderColor::all(PHRASE_RECT_BORDER_COLOR),
                                PhraseSectionRect(i),
                            ))
                            .observe(on_phrase_rect_click)
                            .with_children(|rect| {
                                // Compact, higher-opacity readout of the
                                // same gradient — see PHRASE_ACCENT_ALPHA's
                                // doc comment for why this exists alongside
                                // the full-height fill above.
                                rect.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        width: Val::Percent(100.0),
                                        height: Val::Px(PHRASE_ACCENT_HEIGHT),
                                        ..default()
                                    },
                                    BackgroundColor(phrase_accent_color(learned_frac, false)),
                                    PhraseAccentStripe(i),
                                    Pickable::IGNORE,
                                ));
                            });
                    }
                }

                for note in notes {
                    let Some((left, width)) =
                        note_marker_geometry(note.time, note.duration, duration_secs)
                    else {
                        continue;
                    };
                    let Some((top, height)) = note_lane_geometry(note.hole, hole_count) else {
                        continue;
                    };
                    let fallback = if note.is_blow {
                        NOTE_MARKER_BLOW_FALLBACK
                    } else {
                        NOTE_MARKER_DRAW_FALLBACK
                    };
                    strip.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(left * 100.0),
                            top: Val::Percent(top * 100.0),
                            width: Val::Percent(width * 100.0),
                            height: Val::Percent(height * 100.0),
                            ..default()
                        },
                        BackgroundColor(fallback),
                        NoteMarkerRect {
                            is_blow: note.is_blow,
                        },
                        Pickable::IGNORE,
                    ));
                }
            });

            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.85, 0.35, 0.45)),
                Visibility::Hidden,
                Pickable::IGNORE,
                LoopRangeMarker,
            ));

            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.30, 0.30)),
                Pickable::IGNORE,
                ProgressPlayhead,
            ));
        });
}

/// Sweeps the playhead from the left edge at song start to the right edge at
/// the real end of the audio ([`AudioDuration`], not [`SongEnd`] — see its
/// doc comment). Stays put at the left during the countdown (negative clock)
/// and for looping songs (no finite `SongEnd`); once the real audio content
/// is behind it, clamps at the right edge rather than racing ahead of or
/// lagging the waveform underneath it.
fn update_progress(
    clock: Res<GameplayClock>,
    song_end: Res<SongEnd>,
    duration: Res<AudioDuration>,
    mut playheads: Query<&mut Node, With<ProgressPlayhead>>,
) {
    let progress = if song_end.0.is_finite() && song_end.0 > 0.0 && duration.0 > 0.0 {
        (clock.get() / duration.0).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    for mut node in &mut playheads {
        node.left = Val::Percent(progress * 100.0);
    }
}

/// Left offset and width (fractions 0.0–1.0) for the committed loop
/// marker, or `None` if it shouldn't show — no finite song length, no known
/// audio duration, or the range isn't active. Split out from the system for
/// unit testing without rendering. `song_end_finite` mirrors
/// `update_progress`'s own "does this song have an end" gate;
/// `duration_secs` is [`AudioDuration`], so the marker lines up with the
/// waveform bars it's drawn over.
pub(super) fn loop_marker_geometry(
    active: bool,
    start_time: f64,
    end_time: f64,
    song_end_finite: bool,
    duration_secs: f64,
) -> Option<(f32, f32)> {
    if !active || !song_end_finite || duration_secs <= 0.0 {
        return None;
    }
    let left = (start_time / duration_secs).clamp(0.0, 1.0) as f32;
    let right = (end_time / duration_secs).clamp(0.0, 1.0) as f32;
    Some((left, (right - left).max(0.0)))
}

/// Left offset and width (fractions 0..1) of an in-progress drag preview.
/// Always shown once dragging has started, regardless of width (even a
/// zero-width just-begun drag) — the point is live feedback for what's
/// currently selected, not a judgment call on whether it would validate as a
/// loop yet (that's `loop_range_valid`'s job, applied once the drag ends).
/// `None` only when there's no known audio duration to lay it out against.
pub(super) fn drag_marker_geometry(
    origin_time: f64,
    current_time: f64,
    duration_secs: f64,
) -> Option<(f32, f32)> {
    if duration_secs <= 0.0 {
        return None;
    }
    let a = (origin_time / duration_secs).clamp(0.0, 1.0) as f32;
    let b = (current_time / duration_secs).clamp(0.0, 1.0) as f32;
    let left = a.min(b);
    Some((left, (a.max(b) - left).max(0.0)))
}

/// Top offset and height (fractions 0..1 of the note-lanes strip) for
/// `hole`'s lane, out of `hole_count` total — the highest hole (`hole_count`
/// itself) at the top, the lowest (hole 1) at the bottom, each getting an
/// equal slice. `None` only when there are no holes to divide the strip
/// into (a chart with a zero hole count, which shouldn't happen for a real
/// harmonica but isn't this function's job to rule out). `hole` is clamped
/// into range rather than rejected, so an out-of-range value degrades to
/// the nearest real lane instead of silently dropping the note's marker.
fn note_lane_geometry(hole: u8, hole_count: u8) -> Option<(f32, f32)> {
    if hole_count == 0 {
        return None;
    }
    let lanes = hole_count as f32;
    let lane_from_top = (hole_count - hole.clamp(1, hole_count)) as f32;
    Some((lane_from_top / lanes, 1.0 / lanes))
}

/// The timescale to lay the whole bar out on: `duration_secs` verbatim if
/// there's real audio, else the furthest extent of `notes`/`sections` (a
/// song with no background music has nothing to decode, so `duration_secs`
/// reads `0.0`) — `0.0` only for a truly empty chart.
fn effective_duration(duration_secs: f64, notes: &[NoteMarker], sections: &[PhraseSection]) -> f64 {
    if duration_secs > 0.0 {
        return duration_secs;
    }
    notes
        .iter()
        .map(|n| n.time + n.duration)
        .chain(sections.iter().map(|s| s.end_time))
        .fold(0.0_f64, f64::max)
}

/// Left offset and width (fractions 0..1) for one note's marker — mirrors
/// [`phrase_rect_geometry`]'s shape, but width reflects the note's own
/// duration (matching the Song Editor scrollbar minimap's "note as a
/// proportional rect": `song_editor::interaction::scrollbar_marker`).
/// Floored to [`MIN_NOTE_MARKER_FRAC`] so a very short note still shows as
/// a visible speck, and clamped so a floored marker near the end can't
/// poke past the bar. `None` only when there's no known duration.
fn note_marker_geometry(time: f64, duration: f64, total_duration: f64) -> Option<(f32, f32)> {
    if total_duration <= 0.0 {
        return None;
    }
    let left = (time / total_duration).clamp(0.0, 1.0) as f32;
    let width = (duration / total_duration) as f32;
    Some((left, width.max(MIN_NOTE_MARKER_FRAC).min(1.0 - left)))
}

/// Left offset and width (fractions 0..1) for a phrase section's rectangle,
/// laid out on the same `duration_secs` timescale as the waveform/note
/// markers — same shape as [`loop_marker_geometry`], but unconditional on an
/// "active"/`SongEnd` flag since every section is always shown. `None` only
/// when there's no known audio duration to lay it out against.
fn phrase_rect_geometry(start_time: f64, end_time: f64, duration_secs: f64) -> Option<(f32, f32)> {
    if duration_secs <= 0.0 {
        return None;
    }
    let left = (start_time / duration_secs).clamp(0.0, 1.0) as f32;
    let right = (end_time / duration_secs).clamp(0.0, 1.0) as f32;
    Some((left, (right - left).max(0.0)))
}

/// Low-alpha fill for a phrase-section rectangle — see [`PHRASE_FILL_ALPHA`]
/// for why it's deliberately faint. Color itself comes from
/// `crate::theme::phrase_learned_color`, shared with [`phrase_accent_color`]
/// so the fill and the accent stripe always agree on the same gradient.
fn phrase_fill_color(learned: f32, colorblind: bool) -> Color {
    crate::theme::phrase_learned_color(learned, colorblind).with_alpha(PHRASE_FILL_ALPHA)
}

/// The same gradient as [`phrase_fill_color`], at [`PHRASE_ACCENT_ALPHA`]
/// instead — the compact, legible "how learned is this" readout drawn along
/// a phrase-section rect's top edge (see [`PhraseAccentStripe`]).
fn phrase_accent_color(learned: f32, colorblind: bool) -> Color {
    crate::theme::phrase_learned_color(learned, colorblind).with_alpha(PHRASE_ACCENT_ALPHA)
}

/// Fully opaque border color for every phrase-section rectangle — constant
/// regardless of learned%, so the boundary between sections is always
/// crisp even when two neighbors' fills are nearly the same color.
const PHRASE_RECT_BORDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

/// Border color for whichever section is currently selected (see
/// [`update_selected_phrase_border`]) — a bright gold, distinct from the
/// plain white every other section's border stays.
const SELECTED_PHRASE_RECT_BORDER_COLOR: Color = Color::srgba(1.0, 0.85, 0.25, 1.0);

/// Converts a `RelativeCursorPosition::normalized` reading (-0.5..0.5
/// within the node's bounds; not clamped when the pointer is outside them,
/// which happens constantly mid-drag) into a clamped 0..`duration_secs`
/// time. `None` for an untracked cursor or a non-positive duration.
fn cursor_to_time(normalized_x: Option<f32>, duration_secs: f64) -> Option<f64> {
    let x = normalized_x?;
    if duration_secs <= 0.0 {
        return None;
    }
    let frac = (x as f64 + 0.5).clamp(0.0, 1.0);
    Some(frac * duration_secs)
}

/// Kept in step with `Paused`: the bar is only editable while the game is
/// actually paused — dragging a loop range while notes keep flying by would
/// mean fighting the clock the whole time.
fn sync_progress_bar_mode(paused: Res<Paused>, mut mode: ResMut<ProgressBarMode>) {
    let wanted = if paused.0 {
        ProgressBarMode::Edit
    } else {
        ProgressBarMode::Visualization
    };
    if *mode != wanted {
        *mode = wanted;
    }
}

fn on_drag_start(
    ev: On<Pointer<DragStart>>,
    mode: Res<ProgressBarMode>,
    duration: Res<AudioDuration>,
    surfaces: Query<&RelativeCursorPosition, With<ProgressBarDragSurface>>,
    mut drag: ResMut<LoopDrag>,
) {
    if *mode != ProgressBarMode::Edit || ev.button != PointerButton::Primary {
        return;
    }
    let Ok(rel) = surfaces.get(ev.entity) else {
        return;
    };
    let Some(time) = cursor_to_time(rel.normalized.map(|n| n.x), duration.0) else {
        return;
    };
    *drag = LoopDrag {
        active: true,
        origin_time: time,
        current_time: time,
    };
}

fn on_drag(
    ev: On<Pointer<Drag>>,
    mode: Res<ProgressBarMode>,
    duration: Res<AudioDuration>,
    surfaces: Query<&RelativeCursorPosition, With<ProgressBarDragSurface>>,
    mut drag: ResMut<LoopDrag>,
) {
    if *mode != ProgressBarMode::Edit || !drag.active || ev.button != PointerButton::Primary {
        return;
    }
    let Ok(rel) = surfaces.get(ev.entity) else {
        return;
    };
    if let Some(time) = cursor_to_time(rel.normalized.map(|n| n.x), duration.0) {
        drag.current_time = time;
    }
}

/// Releasing the mouse ends the drag and requests the range it swept out —
/// see [`RequestLoopRange`]'s doc comment for why this doesn't just write
/// `LoopConfig` directly.
fn on_drag_end(
    ev: On<Pointer<DragEnd>>,
    mut drag: ResMut<LoopDrag>,
    mut requests: MessageWriter<RequestLoopRange>,
) {
    if !drag.active || ev.button != PointerButton::Primary {
        return;
    }
    drag.active = false;
    requests.write(RequestLoopRange {
        start_time: drag.origin_time.min(drag.current_time),
        end_time: drag.origin_time.max(drag.current_time),
    });
}

/// Applies a requested loop range to `LoopConfig`. `loop_range_valid` decides
/// `active`, so a degenerate (zero-width) drag cleanly ends up inactive
/// rather than a stale range with a confusing on-screen marker.
fn apply_requested_loop_range(
    mut requests: MessageReader<RequestLoopRange>,
    mut loop_cfg: ResMut<LoopConfig>,
) {
    for req in requests.read() {
        loop_cfg.start_time = req.start_time;
        loop_cfg.end_time = req.end_time;
        loop_cfg.active = loop_range_valid(req.start_time, req.end_time);
    }
}

/// Keeps the loop marker in step with either an in-progress drag (live
/// preview, takes priority) or the committed `LoopConfig` (once the drag
/// ends and `apply_requested_loop_range` has caught up — ordered `.before`
/// this system so both happen in the same frame). Not gated on `Paused` for
/// the `LoopConfig` branch — a range set before pausing again should still
/// render — but dragging itself is only ever possible while paused.
fn update_loop_marker(
    loop_cfg: Res<LoopConfig>,
    song_end: Res<SongEnd>,
    duration: Res<AudioDuration>,
    drag: Res<LoopDrag>,
    mut markers: Query<(&mut Node, &mut Visibility), With<LoopRangeMarker>>,
) {
    if !loop_cfg.is_changed() && !song_end.is_changed() && !drag.is_changed() {
        return;
    }
    let geometry = if drag.active {
        drag_marker_geometry(drag.origin_time, drag.current_time, duration.0)
    } else {
        loop_marker_geometry(
            loop_cfg.active,
            loop_cfg.start_time,
            loop_cfg.end_time,
            song_end.0.is_finite() && song_end.0 > 0.0,
            duration.0,
        )
    };
    for (mut node, mut vis) in &mut markers {
        match geometry {
            Some((left, width)) => {
                node.left = Val::Percent(left * 100.0);
                node.width = Val::Percent(width * 100.0);
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// Re-tints each phrase-section rectangle (fill and accent stripe alike)
/// when `AdaptiveDifficulty` or `ColorblindPalette` changes (a manual
/// pause-menu edit, a fresh song's initial load, or flipping the colorblind
/// setting mid-song) without respawning — the rectangles themselves are
/// only ever (re)created in [`spawn_song_progress`], since their
/// count/geometry only changes when the song itself does.
fn update_phrase_section_colors(
    adaptive: Res<AdaptiveDifficulty>,
    colorblind: Res<crate::settings::ColorblindPalette>,
    mut rects: Query<(&PhraseSectionRect, &mut BackgroundColor), Without<PhraseAccentStripe>>,
    mut stripes: Query<(&PhraseAccentStripe, &mut BackgroundColor), Without<PhraseSectionRect>>,
) {
    if !adaptive.is_changed() && !colorblind.is_changed() {
        return;
    }
    for (rect, mut color) in &mut rects {
        let learned = adaptive.learned.get(rect.0).copied().unwrap_or(0.0);
        *color = BackgroundColor(phrase_fill_color(learned, colorblind.0));
    }
    for (stripe, mut color) in &mut stripes {
        let learned = adaptive.learned.get(stripe.0).copied().unwrap_or(0.0);
        *color = BackgroundColor(phrase_accent_color(learned, colorblind.0));
    }
}

/// Keeps every note-marker rect's color matching whatever the active theme
/// (or, with `ColorblindPalette` on, the fixed colorblind pair) currently
/// uses for blow/draw — the same source `gameplay_2d::spawn_visible_notes`
/// tints the falling notes from, via the same `crate::theme::
/// effective_note_colors`, so the bar's markers never drift from what the
/// rest of the screen shows. Runs unconditionally each frame (cheap — a
/// song has at most a few dozen markers visible at once) rather than
/// gating on a change check, the same "just re-sync every currently-
/// spawned one" approach `gameplay_2d`'s own `update_note_visuals*`
/// systems use, which sidesteps needing an `Added`-vs-`Changed` split to
/// catch both a fresh song's new markers and a live theme/setting change.
fn sync_note_marker_colors(
    theme: Res<crate::theme::LoadedTheme>,
    colorblind: Res<crate::settings::ColorblindPalette>,
    mut markers: Query<(&NoteMarkerRect, &mut BackgroundColor)>,
) {
    let colors = crate::theme::effective_note_colors(theme.note_colors(), colorblind.0);
    for (marker, mut color) in &mut markers {
        let base = if marker.is_blow {
            colors.blow
        } else {
            colors.draw
        };
        *color = BackgroundColor(base.with_alpha(NOTE_MARKER_ALPHA));
    }
}

/// The pause menu's phrase selector is now driven by clicking a section's
/// rectangle here, not prev/next buttons — see `pause_menu::
/// SelectedPhraseIndex`'s doc comment. Only while paused, matching every
/// other interactive behaviour this bar has (dragging a loop range).
fn on_phrase_rect_click(
    ev: On<Pointer<Click>>,
    rects: Query<&PhraseSectionRect>,
    mode: Res<ProgressBarMode>,
    mut selected: ResMut<SelectedPhraseIndex>,
) {
    if *mode != ProgressBarMode::Edit {
        return;
    }
    let Ok(rect) = rects.get(ev.entity) else {
        return;
    };
    selected.0 = rect.0;
}

/// A brighter border on the currently-selected phrase-section rectangle —
/// the only on-bar confirmation of which section the pause menu's "Learned"
/// slider will edit. Re-applied not just when `SelectedPhraseIndex` changes
/// but whenever fresh rectangles just appeared (a new song's bar spawned):
/// `SelectedPhraseIndex` deliberately isn't reset on a new song (see its
/// own doc comment), so it wouldn't otherwise read as "changed" once
/// there's something new to paint it onto.
fn update_selected_phrase_border(
    selected: Res<SelectedPhraseIndex>,
    mut rects: Query<(&PhraseSectionRect, &mut BorderColor)>,
    fresh_rects: Query<(), Added<PhraseSectionRect>>,
) {
    if !selected.is_changed() && fresh_rects.is_empty() {
        return;
    }
    for (rect, mut border) in &mut rects {
        *border = BorderColor::all(if rect.0 == selected.0 {
            SELECTED_PHRASE_RECT_BORDER_COLOR
        } else {
            PHRASE_RECT_BORDER_COLOR
        });
    }
}

pub struct SongProgressPlugin;

impl Plugin for SongProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SongWaveformMaterialPlugin)
            .init_resource::<AudioDuration>()
            .init_resource::<ProgressBarMode>()
            .init_resource::<LoopDrag>()
            .add_message::<RequestLoopRange>()
            .add_systems(
                Update,
                update_progress.run_if(in_state(AppState::Playing).and_then(|p: Res<Paused>| !p.0)),
            )
            .add_systems(
                Update,
                (
                    sync_progress_bar_mode,
                    apply_requested_loop_range,
                    update_loop_marker,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    update_phrase_section_colors,
                    update_selected_phrase_border,
                    sync_note_marker_colors,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── note_marker_geometry ───────────────────────────────────────────────────

    #[test]
    fn note_marker_hidden_without_a_known_duration() {
        assert_eq!(note_marker_geometry(0.0, 1.0, 0.0), None);
    }

    #[test]
    fn note_marker_geometry_is_a_fraction_of_the_total_duration() {
        // A 4s note starting at 8s of a 64s song: left an eighth of the way
        // in, a sixteenth wide.
        let (left, width) = note_marker_geometry(8.0, 4.0, 64.0).unwrap();
        assert!((left - 0.125).abs() < 1e-6);
        assert!((width - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn note_marker_floors_the_width_of_a_very_short_note() {
        // A near-instantaneous note in a long song would be invisibly thin
        // without the floor.
        let (_, width) = note_marker_geometry(0.0, 0.001, 10_000.0).unwrap();
        assert!(width >= MIN_NOTE_MARKER_FRAC);
    }

    #[test]
    fn note_marker_never_pokes_past_the_bars_end() {
        // A floored marker on the song's very last instant must stay
        // within the bar (left + width <= 1.0).
        let (left, width) = note_marker_geometry(9_999.999, 0.0001, 10_000.0).unwrap();
        assert!(left + width <= 1.0);
    }

    #[test]
    fn note_marker_geometry_handles_a_note_spanning_the_whole_song() {
        // A note whose own duration equals the whole song (a degenerate but
        // possible input) shouldn't panic or invert.
        let (left, width) = note_marker_geometry(0.0, 10.0, 10.0).unwrap();
        assert_eq!(left, 0.0);
        assert!((width - 1.0).abs() < 1e-6);
    }

    // ── note_lane_geometry ─────────────────────────────────────────────────────

    #[test]
    fn note_lane_hidden_with_a_zero_hole_count() {
        assert_eq!(note_lane_geometry(1, 0), None);
    }

    #[test]
    fn highest_hole_is_the_top_lane() {
        let (top, height) = note_lane_geometry(10, 10).unwrap();
        assert_eq!(top, 0.0);
        assert!((height - 0.1).abs() < 1e-6);
    }

    #[test]
    fn lowest_hole_is_the_bottom_lane() {
        // Hole 1 of 10 is the *last* lane (bottom), not the first —
        // opposite the Song Editor's own scrollbar minimap ordering.
        let (top, height) = note_lane_geometry(1, 10).unwrap();
        assert!((top - 0.9).abs() < 1e-6);
        assert!((height - 0.1).abs() < 1e-6);
    }

    #[test]
    fn middle_hole_lands_in_its_own_slice() {
        // Hole 5 of 10: 5 lanes down from the highest (holes 10..6 above
        // it), so lane_from_top = 10 - 5 = 5.
        let (top, height) = note_lane_geometry(5, 10).unwrap();
        assert!((top - 0.5).abs() < 1e-6);
        assert!((height - 0.1).abs() < 1e-6);
    }

    #[test]
    fn lanes_are_narrower_on_a_chromatic_harp() {
        let (_, height_10) = note_lane_geometry(1, 10).unwrap();
        let (_, height_12) = note_lane_geometry(1, 12).unwrap();
        assert!(height_12 < height_10);
    }

    #[test]
    fn note_lane_geometry_clamps_an_out_of_range_hole() {
        // Defensive: a hole past hole_count degrades to the top lane
        // rather than vanishing the marker entirely.
        assert_eq!(note_lane_geometry(99, 10), note_lane_geometry(10, 10));
    }

    // ── effective_duration (the no-background-music fallback) ────────────────

    fn marker(time: f64, duration: f64) -> NoteMarker {
        NoteMarker {
            time,
            duration,
            hole: 1,
            is_blow: true,
        }
    }

    fn section(start_time: f64, end_time: f64) -> PhraseSection {
        PhraseSection {
            name: String::new(),
            start_time,
            end_time,
            note_count: 0,
        }
    }

    #[test]
    fn effective_duration_prefers_real_audio_duration_when_present() {
        let notes = [marker(0.0, 1.0), marker(50.0, 1.0)];
        assert_eq!(effective_duration(64.0, &notes, &[]), 64.0);
    }

    #[test]
    fn effective_duration_falls_back_to_the_furthest_note_end_without_music() {
        let notes = [marker(0.0, 1.0), marker(20.0, 4.0), marker(5.0, 1.0)];
        assert_eq!(effective_duration(0.0, &notes, &[]), 24.0);
    }

    #[test]
    fn effective_duration_falls_back_to_the_furthest_phrase_section_too() {
        // A phrase section can extend past every individual note (e.g. a
        // trailing rest) — the fallback must cover it too, not just notes.
        let notes = [marker(0.0, 1.0)];
        let sections = [section(0.0, 30.0)];
        assert_eq!(effective_duration(0.0, &notes, &sections), 30.0);
    }

    #[test]
    fn effective_duration_is_zero_for_a_truly_empty_chart() {
        assert_eq!(effective_duration(0.0, &[], &[]), 0.0);
    }

    // ── phrase_rect_geometry / phrase_fill_color ──────────────────────────────

    #[test]
    fn phrase_rect_hidden_without_a_known_audio_duration() {
        assert_eq!(phrase_rect_geometry(0.0, 8.0, 0.0), None);
    }

    #[test]
    fn phrase_rect_geometry_is_a_fraction_of_the_audio_duration() {
        let (left, width) = phrase_rect_geometry(8.0, 16.0, 64.0).unwrap();
        assert!((left - 0.125).abs() < 1e-6);
        assert!((width - 0.125).abs() < 1e-6);
    }

    #[test]
    fn phrase_fill_color_matches_the_theme_gradient_at_the_given_alpha() {
        // The gradient itself (endpoints, colorblind swap, clamping) is
        // `theme::phrase_learned_color`'s own responsibility and already
        // covered by its unit tests — this just confirms the fill/accent
        // wrappers apply the right alpha on top of whatever that returns.
        for (learned, colorblind) in [(0.0, false), (1.0, false), (0.4, true)] {
            let expected_hue = crate::theme::phrase_learned_color(learned, colorblind);
            let Color::Srgba(fill) = phrase_fill_color(learned, colorblind) else {
                panic!("expected Srgba");
            };
            let Color::Srgba(accent) = phrase_accent_color(learned, colorblind) else {
                panic!("expected Srgba");
            };
            let Color::Srgba(expected) = expected_hue else {
                panic!("expected Srgba");
            };
            assert!((fill.red - expected.red).abs() < 1e-6);
            assert!((fill.alpha - PHRASE_FILL_ALPHA).abs() < 1e-6);
            assert!((accent.red - expected.red).abs() < 1e-6);
            assert!((accent.alpha - PHRASE_ACCENT_ALPHA).abs() < 1e-6);
        }
    }

    #[test]
    fn phrase_fill_color_clamps_out_of_range_input() {
        assert_eq!(
            phrase_fill_color(-1.0, false),
            phrase_fill_color(0.0, false)
        );
        assert_eq!(phrase_fill_color(2.0, false), phrase_fill_color(1.0, false));
    }

    // ── loop_marker_geometry (committed LoopConfig) ───────────────────────────

    #[test]
    fn loop_marker_hidden_when_inactive() {
        assert_eq!(loop_marker_geometry(false, 8.0, 16.0, true, 60.0), None);
    }

    #[test]
    fn loop_marker_hidden_without_a_finite_song_end() {
        assert_eq!(loop_marker_geometry(true, 8.0, 16.0, false, 60.0), None);
    }

    #[test]
    fn loop_marker_hidden_without_a_known_audio_duration() {
        assert_eq!(loop_marker_geometry(true, 8.0, 16.0, true, 0.0), None);
    }

    #[test]
    fn loop_marker_geometry_is_a_fraction_of_the_audio_duration() {
        // 8s..16s of a 64s track → starts an eighth of the way in, an
        // eighth wide — driven by the real audio duration, not `SongEnd`.
        let (left, width) = loop_marker_geometry(true, 8.0, 16.0, true, 64.0).unwrap();
        assert!((left - 0.125).abs() < 1e-6);
        assert!((width - 0.125).abs() < 1e-6);
    }

    // ── drag_marker_geometry (live drag preview) ──────────────────────────────

    #[test]
    fn drag_marker_hidden_without_a_known_audio_duration() {
        assert_eq!(drag_marker_geometry(4.0, 8.0, 0.0), None);
    }

    #[test]
    fn drag_marker_geometry_normalizes_direction() {
        // Dragging right-to-left (current before origin) should still yield
        // the same left/width as dragging left-to-right.
        let forward = drag_marker_geometry(8.0, 16.0, 64.0).unwrap();
        let backward = drag_marker_geometry(16.0, 8.0, 64.0).unwrap();
        assert_eq!(forward, backward);
        assert!((forward.0 - 0.125).abs() < 1e-6);
        assert!((forward.1 - 0.125).abs() < 1e-6);
    }

    #[test]
    fn drag_marker_geometry_shows_a_zero_width_preview_at_drag_start() {
        let (left, width) = drag_marker_geometry(10.0, 10.0, 64.0).unwrap();
        assert!((left - 10.0 / 64.0).abs() < 1e-6);
        assert_eq!(width, 0.0);
    }

    // ── cursor_to_time ─────────────────────────────────────────────────────────

    #[test]
    fn cursor_to_time_is_none_without_a_tracked_cursor() {
        assert_eq!(cursor_to_time(None, 60.0), None);
    }

    #[test]
    fn cursor_to_time_is_none_without_a_positive_duration() {
        assert_eq!(cursor_to_time(Some(0.0), 0.0), None);
    }

    #[test]
    fn cursor_to_time_maps_the_node_span_to_the_full_duration() {
        // -0.5 (left edge) .. 0.5 (right edge) maps to 0..duration.
        assert_eq!(cursor_to_time(Some(-0.5), 60.0), Some(0.0));
        assert_eq!(cursor_to_time(Some(0.5), 60.0), Some(60.0));
        assert_eq!(cursor_to_time(Some(0.0), 60.0), Some(30.0));
    }

    #[test]
    fn cursor_to_time_clamps_outside_the_node_bounds() {
        // Dragging past either edge of the bar clamps to the endpoints
        // instead of extrapolating past 0 or the duration.
        assert_eq!(cursor_to_time(Some(-2.0), 60.0), Some(0.0));
        assert_eq!(cursor_to_time(Some(2.0), 60.0), Some(60.0));
    }

    // ── apply_requested_loop_range ─────────────────────────────────────────────

    #[test]
    fn apply_requested_loop_range_activates_a_valid_range() {
        let mut world = World::new();
        world.insert_resource(LoopConfig::default());
        world.init_resource::<Messages<RequestLoopRange>>();
        world.write_message(RequestLoopRange {
            start_time: 8.0,
            end_time: 16.0,
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_requested_loop_range);
        schedule.run(&mut world);
        let cfg = world.resource::<LoopConfig>();
        assert!(cfg.active);
        assert_eq!(cfg.start_time, 8.0);
        assert_eq!(cfg.end_time, 16.0);
    }

    #[test]
    fn apply_requested_loop_range_leaves_a_degenerate_range_inactive() {
        let mut world = World::new();
        world.insert_resource(LoopConfig::default());
        world.init_resource::<Messages<RequestLoopRange>>();
        world.write_message(RequestLoopRange {
            start_time: 8.0,
            end_time: 8.0,
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_requested_loop_range);
        schedule.run(&mut world);
        assert!(!world.resource::<LoopConfig>().active);
    }

    // ── sync_progress_bar_mode ─────────────────────────────────────────────────

    #[test]
    fn sync_progress_bar_mode_follows_paused() {
        let mut world = World::new();
        world.insert_resource(Paused(true));
        world.insert_resource(ProgressBarMode::Visualization);
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_progress_bar_mode);
        schedule.run(&mut world);
        assert_eq!(*world.resource::<ProgressBarMode>(), ProgressBarMode::Edit);

        world.insert_resource(Paused(false));
        schedule.run(&mut world);
        assert_eq!(
            *world.resource::<ProgressBarMode>(),
            ProgressBarMode::Visualization
        );
    }
}
