// SPDX-License-Identifier: MIT

//! The mod panel's two-strip assembly: a short, fixed global-transport strip
//! (Back / Edit / Perform / Lock / Save / Load — always the same regardless
//! of mode), then a `flex_wrap: Wrap` contextual tool strip below it (the
//! current mode's whole tool palette). See [`spawn_mod_panel`]'s doc comment
//! for why it's two stacked rows rather than one ever-growing row. Built
//! from the reusable button shapes in `super::panel_widgets`.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::AppState;
use super::panel_widgets::{
    mod_button, mode_button, panel_separator, timeline_tool_button, transport_button,
};
use super::playback::{EditorAudio, Playhead};
use super::practice::{PracticeState, stop_practice};
use super::record::{RecordState, stop_record};
use super::state::{EditorState, Mode, TimelineTool};
use super::transport::{spawn_file_buttons, spawn_playback_buttons, spawn_record_buttons};
use super::ui::{
    EditModeGroup, ModButton, ModeButton, PlayModeGroup, RecordModeGroup, TimelineToolButton,
};
use crate::audio_system::pitch_detect::{PitchAlgorithm, PitchRange};
use crate::dialogs::algo_picker::{algo_labels, attach_algo_tooltip, on_algo_selected};
use crate::dialogs::combobox;
use crate::localization::LocalizationExt;
use crate::settings::ActionButtonStyle;
use crate::theme::SongEditorColors;
use bevy_fluent::prelude::Localization;

/// The mod panel: a short, fixed global-transport strip (Back / Edit /
/// Perform / Lock / Save / Load — always the same regardless of mode), then
/// a `flex_wrap: Wrap` contextual tool strip below it (the current mode's
/// whole tool palette — up to 13 buttons + 3 separators in Edit mode). Two
/// stacked rows rather than one ever-growing row, so a narrow/small window
/// wraps the tool strip onto a second line instead of rendering buttons past
/// the right edge with no way to reach them. The panel's own height is
/// therefore auto (driven by its two rows' content) rather than the fixed
/// `Val::Px(52.0)` a single non-wrapping row could get away with.
pub(super) fn spawn_mod_panel(
    root: &mut ChildSpawnerCommands,
    loc: &Localization,
    colors: SongEditorColors,
    mode: Mode,
    editor_root: Entity,
    algorithm: PitchAlgorithm,
    style: ActionButtonStyle,
) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(colors.panel_bg),
    ))
    .with_children(|panel| {
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|transport| {
                transport_button(
                    transport,
                    loc.msg("editor-back-label"),
                    loc.msg("editor-back-tooltip"),
                    "\u{2190}",
                    style,
                    colors.transport_back,
                    |_: On<Activate>,
                     mut next: ResMut<NextState<AppState>>,
                     mut ret_play: ResMut<crate::app::ReturnToPlay>| {
                        ret_play.0 = true;
                        next.set(AppState::Menu);
                    },
                );
                panel_separator(transport);

                // Edit/Record/Play/Lock: always visible, regardless of
                // which mode-group below is currently shown. Every mode
                // switch stops whatever the departed mode had running —
                // its transport is about to disappear, so nothing would be
                // left to stop it.
                mode_button(
                    transport,
                    ModeButton::Edit,
                    loc.msg("editor-mode-edit"),
                    loc.msg("editor-mode-edit-tooltip"),
                    "\u{270E}",
                    style,
                    colors,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     playing: Query<Entity, With<EditorAudio>>,
                     mut practice: ResMut<PracticeState>,
                     mut record: ResMut<RecordState>,
                     mut playhead: ResMut<Playhead>,
                     mut pitch_range: ResMut<PitchRange>,
                     mut count_in: ResMut<super::metronome::CountIn>,
                     mut commands: Commands| {
                        state.mode = Mode::Edit;
                        stop_practice(&playing, &mut practice, &mut playhead, &mut commands);
                        stop_record(
                            &mut state,
                            &playing,
                            &mut record,
                            &mut playhead,
                            &mut pitch_range,
                            &mut count_in,
                            &mut commands,
                        );
                    },
                );
                mode_button(
                    transport,
                    ModeButton::Record,
                    loc.msg("editor-mode-record"),
                    loc.msg("editor-mode-record-tooltip"),
                    "\u{23FA}",
                    style,
                    colors,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     playing: Query<Entity, With<EditorAudio>>,
                     mut practice: ResMut<PracticeState>,
                     mut playhead: ResMut<Playhead>,
                     mut commands: Commands| {
                        state.mode = Mode::Record;
                        // A recording can only have been started from this
                        // mode itself, so only Play-mode playback/practice
                        // needs stopping here.
                        stop_practice(&playing, &mut practice, &mut playhead, &mut commands);
                    },
                );
                mode_button(
                    transport,
                    ModeButton::Play,
                    loc.msg("editor-mode-play"),
                    loc.msg("editor-mode-play-tooltip"),
                    "\u{1F3B5}",
                    style,
                    colors,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     playing: Query<Entity, With<EditorAudio>>,
                     mut record: ResMut<RecordState>,
                     mut playhead: ResMut<Playhead>,
                     mut pitch_range: ResMut<PitchRange>,
                     mut count_in: ResMut<super::metronome::CountIn>,
                     mut commands: Commands| {
                        state.mode = Mode::Play;
                        stop_record(
                            &mut state,
                            &playing,
                            &mut record,
                            &mut playhead,
                            &mut pitch_range,
                            &mut count_in,
                            &mut commands,
                        );
                    },
                );
                mode_button(
                    transport,
                    ModeButton::Lock,
                    loc.msg("editor-lock"),
                    loc.msg("editor-lock-tooltip"),
                    "\u{1F512}",
                    style,
                    colors,
                    |_: On<Activate>, mut state: ResMut<EditorState>| {
                        state.user_locked = !state.user_locked;
                    },
                );

                panel_separator(transport);

                // Undo/redo are plain click actions (not a mode toggle),
                // so `transport_button` rather than `mode_button` — and a
                // no-op rather than visually disabled when the relevant
                // stack is empty, the same "clicking does nothing" shape
                // `UndoHistory::undo`/`redo` already have. See
                // `undo::UndoHistory`'s doc comment for exactly what
                // counts as an undoable edit.
                transport_button(
                    transport,
                    loc.msg("editor-undo"),
                    loc.msg("editor-undo-tooltip"),
                    "\u{21B6}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     mut history: ResMut<super::undo::UndoHistory>| {
                        history.undo(&mut state);
                    },
                )
                .insert(super::ui::UndoRedoButton::Undo);
                transport_button(
                    transport,
                    loc.msg("editor-redo"),
                    loc.msg("editor-redo-tooltip"),
                    "\u{21B7}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     mut history: ResMut<super::undo::UndoHistory>| {
                        history.redo(&mut state);
                    },
                )
                .insert(super::ui::UndoRedoButton::Redo);

                // On-screen equivalents of Delete/Backspace and Ctrl+C/
                // Ctrl+V (`interaction::grid_keys`/`handle_copy_paste`) —
                // those are the *only* way to delete/copy/paste on a
                // touch-only device with no keyboard. Paste has no cursor
                // position to anchor on without a mouse, so it lands at the
                // start of the current view (`state.scroll_beat`) instead
                // of "wherever the mouse is," unlike the keyboard shortcut.
                transport_button(
                    transport,
                    loc.msg("editor-delete"),
                    loc.msg("editor-delete-tooltip"),
                    "\u{2717}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>, mut state: ResMut<EditorState>| {
                        super::interaction::delete_selected(&mut state);
                    },
                );
                transport_button(
                    transport,
                    loc.msg("editor-copy"),
                    loc.msg("editor-copy-tooltip"),
                    "\u{25C8}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>,
                     state: Res<EditorState>,
                     mut clipboard: ResMut<super::clipboard::NoteClipboard>| {
                        if !state.selected.is_empty() {
                            clipboard.0 = super::clipboard::copy_selected(&state.notes, &state.selected);
                        }
                    },
                );
                transport_button(
                    transport,
                    loc.msg("editor-paste"),
                    loc.msg("editor-paste-tooltip"),
                    "\u{21B4}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>,
                     mut state: ResMut<EditorState>,
                     clipboard: Res<super::clipboard::NoteClipboard>| {
                        if clipboard.0.is_empty() {
                            return;
                        }
                        let tick = state.scroll_beat * super::TICKS_PER_BEAT;
                        let hole_count = state.hole_count();
                        let (pasted, next_id) = super::clipboard::paste_targets(
                            &clipboard.0,
                            tick,
                            hole_count,
                            &state.notes,
                            state.next_id,
                        );
                        if !pasted.is_empty() {
                            state.next_id = next_id;
                            state.selected = pasted.iter().map(|n| n.id).collect();
                            state.notes.extend(pasted);
                        }
                    },
                );

                // The metronome click, shared with gameplay/the Bending
                // Trainer via the same `MetronomeMuted` global (see
                // `metronome`'s module doc) — clicks during Record/Play/
                // Practice, dimmed here while muted rather than a
                // live-swapped label, same visual language as Undo/Redo.
                transport_button(
                    transport,
                    loc.msg("editor-metronome"),
                    loc.msg("editor-metronome-tooltip"),
                    "\u{1F514}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>,
                     mut muted: ResMut<crate::gameplay::metronome_overlay::MetronomeMuted>| {
                        muted.0 = !muted.0;
                    },
                )
                .insert(super::ui::MetronomeToggleButton);

                // Toggles the meta form's third (color-legend) column — see
                // `meta_form::update_legend_visibility`.
                transport_button(
                    transport,
                    loc.msg("editor-legend-toggle"),
                    loc.msg("editor-legend-toggle-tooltip"),
                    "\u{2139}",
                    style,
                    colors.btn_bg,
                    |_: On<Activate>, mut state: ResMut<EditorState>| {
                        state.legend_visible = !state.legend_visible;
                    },
                );

                // Dev-only ("--features dev") benchmark ground-truth mode —
                // see `expected_notes`'s own module docs.
                #[cfg(feature = "dev")]
                super::expected_notes::spawn_expected_notes_mode_button(transport, loc, colors, style);

                panel_separator(transport);

                spawn_file_buttons(transport, loc, colors, style);

                // Dev-only debugging aid — see `debug_record`'s own module
                // docs. Deliberately in this always-visible strip, not a
                // mode-specific group: it needs to stay checkable (and its
                // one shared checkbox/status-label entity needs to exist
                // exactly once) regardless of whether the mic tap it arms
                // ends up gated on `RecordState::active` or
                // `PracticeState::active` — see `sync_raw_capture`.
                #[cfg(feature = "dev")]
                super::debug_record::spawn_debug_recording_controls(transport, loc, colors, style);
            });

        panel
            .spawn((
                EditModeGroup,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(6.0),
                    // `Display::None`, not `Visibility::Hidden` — Visibility
                    // only skips rendering, it still reserves this group's
                    // full layout width, which pushed the other group off to
                    // the right instead of freeing its place.
                    display: if mode == Mode::Edit {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
            ))
            .with_children(|g| {
                mod_button(
                    g,
                    ModButton::Blow,
                    loc.msg("mod-blow"),
                    loc.msg("mod-blow-tooltip"),
                    "\u{2191}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Draw,
                    loc.msg("mod-draw"),
                    loc.msg("mod-draw-tooltip"),
                    "\u{2193}",
                    style,
                    colors,
                );
                panel_separator(g);
                mod_button(
                    g,
                    ModButton::Bend,
                    loc.msg("mod-bend"),
                    loc.msg("mod-bend-tooltip"),
                    "\u{007E}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Overblow,
                    loc.msg("mod-overblow"),
                    loc.msg("mod-overblow-tooltip"),
                    "\u{21C8}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Overdraw,
                    loc.msg("mod-overdraw"),
                    loc.msg("mod-overdraw-tooltip"),
                    "\u{21CA}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Slide,
                    loc.msg("mod-slide"),
                    loc.msg("mod-slide-tooltip"),
                    "\u{2194}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Wah,
                    loc.msg("mod-wah"),
                    loc.msg("mod-wah-tooltip"),
                    "\u{2248}",
                    style,
                    colors,
                );
                mod_button(
                    g,
                    ModButton::Vibrato,
                    loc.msg("mod-vibrato"),
                    loc.msg("mod-vibrato-tooltip"),
                    "\u{2195}",
                    style,
                    colors,
                );
                g.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                mod_button(
                    g,
                    ModButton::Delete,
                    loc.msg("mod-delete"),
                    loc.msg("mod-delete-tooltip"),
                    "\u{25CB}",
                    style,
                    colors,
                );
                panel_separator(g);
                timeline_tool_button(
                    g,
                    TimelineToolButton(TimelineTool::Select),
                    loc.msg("editor-tool-select"),
                    loc.msg("editor-tool-select-tooltip"),
                    "\u{25FB}",
                    style,
                    colors,
                );
                timeline_tool_button(
                    g,
                    TimelineToolButton(TimelineTool::Erase),
                    loc.msg("editor-tool-erase"),
                    loc.msg("editor-tool-erase-tooltip"),
                    "\u{25AD}",
                    style,
                    colors,
                );
                timeline_tool_button(
                    g,
                    TimelineToolButton(TimelineTool::Remove),
                    loc.msg("editor-tool-remove"),
                    loc.msg("editor-tool-remove-tooltip"),
                    "\u{25FC}",
                    style,
                    colors,
                );
                timeline_tool_button(
                    g,
                    TimelineToolButton(TimelineTool::Tempo),
                    loc.msg("editor-tool-tempo"),
                    loc.msg("editor-tool-tempo-tooltip"),
                    "\u{2669}",
                    style,
                    colors,
                );
            });

        let mut record_group_ec = panel.spawn((
            RecordModeGroup,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(6.0),
                display: if mode == Mode::Record {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
        ));
        // Captured so the combobox below can use it as its own trigger
        // parent — `combobox::spawn_combobox` needs a concrete `Entity` up
        // front, and this row (unlike `EditorRoot`) is spawned fresh right
        // here, so there's nothing to query for.
        let record_group_id = record_group_ec.id();
        record_group_ec.with_children(|g| {
            spawn_record_buttons(g, loc, colors, style);

            // Detect algorithm: same shared combobox (and global
            // `AudioSettings::pitch_algorithm`) as Options/Bending Trainer —
            // picking one here takes effect immediately, including for a
            // take already in progress, since recording reads pitches off
            // the same continuously-running mic pipeline every other mode
            // does (see `record.rs`'s module docs).
            let algo_combo = combobox::spawn_combobox(
                g.commands_mut(),
                record_group_id,
                editor_root,
                &loc.msg("editor-record-detect-label"),
                &algo_labels(),
                algorithm.label(),
                on_algo_selected,
            );
            attach_algo_tooltip(g.commands_mut(), algo_combo, algorithm);
        });

        panel
            .spawn((
                PlayModeGroup,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(6.0),
                    display: if mode == Mode::Play {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
            ))
            .with_children(|g| {
                spawn_playback_buttons(g, loc, colors, style);
            });

        // Dev-only ("--features dev") benchmark ground-truth mode — see
        // `expected_notes`'s own module docs.
        #[cfg(feature = "dev")]
        super::expected_notes::spawn_expected_notes_group(panel, loc, colors, mode, style);
    });
}
