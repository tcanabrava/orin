// SPDX-License-Identifier: MIT

//! Per-track mute row for a MIDI-backed Jam Session song
//! (`song::MidiTrackAudio`): each track plays as its own simultaneous,
//! synchronized `AudioSink` (spawned by `gameplay::countdown_overlay::
//! update_countdown`), so muting one is just zeroing that sink's volume —
//! no live re-mixing needed, since every stem is already a complete,
//! independent render (`song::midi::render_track_pcm`).

use bevy::audio::{AudioSink, Volume};
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::ui_widgets::Button as WidgetButton;
use bevy_fluent::Localization;

use crate::dialogs::tooltip::Tooltip;
use crate::gameplay::MidiTrackPlayer;
use harmonicon_audio::AudioSettings;
use harmonicon_platform::localization::LocalizationExt;
use harmonicon_song::song::MidiTrackAudio;

/// Per-track mute state for the currently-playing MIDI-backed song — index
/// matches `SongManifest::midi_tracks`. Sized (and reset to all-unmuted) by
/// `jam::session::setup` for every jam, whether or not the song actually
/// has MIDI tracks (empty otherwise, so the systems below are cheap no-ops
/// for an ordinary song — nothing to iterate).
#[derive(Resource, Default)]
pub struct JamMidiMute(pub Vec<bool>);

/// Tags one mute-toggle button with which track it controls, so one shared
/// `toggle_track_mute` observer (cloned onto every button) can look up
/// which track fired via the clicked entity — see `gameplay::
/// harmonica_overlay::DiagramCellTarget` for the same pattern.
#[derive(Component, Clone, Copy)]
pub struct TrackMuteCell(usize);

/// The "with sound"/"no sound" icon inside one mute button — its text is
/// the only part [`update_track_mute_buttons`] rewrites; the track-name
/// label next to it is static, set once at spawn.
#[derive(Component, Clone, Copy)]
pub struct TrackMuteIcon(usize);

const SOUND_ICON: &str = "\u{1F50A}"; // 🔊
const MUTED_ICON: &str = "\u{1F507}"; // 🔇
const MUTED_BG: Color = Color::srgb(0.30, 0.14, 0.14);
const UNMUTED_BG: Color = Color::srgb(0.16, 0.30, 0.18);

/// A horizontal row of per-track mute-toggle buttons, one per
/// `SongManifest::midi_tracks` entry — only call this when the current
/// song actually has MIDI tracks (`jam::session::setup` checks first).
pub fn spawn_midi_track_row(
    parent: &mut ChildSpawnerCommands,
    tracks: &[MidiTrackAudio],
    loc: &Localization,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .with_children(|row| {
            for (index, track) in tracks.iter().enumerate() {
                row.spawn((
                    WidgetButton,
                    TabIndex(0),
                    TrackMuteCell(index),
                    Node {
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(UNMUTED_BG),
                    BorderColor::all(Color::srgb(0.30, 0.30, 0.40)),
                    Tooltip(String::from(loc.msg("jam-midi-track-mute-tooltip"))),
                ))
                .observe(toggle_track_mute)
                .with_children(|b| {
                    b.spawn((
                        TrackMuteIcon(index),
                        Text::new(SOUND_ICON),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                    b.spawn((
                        Text::new(format!(" {}", track.name)),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

/// Shared across every mute button; looks up which track fired via
/// `TrackMuteCell` on the clicked entity rather than a per-button closure.
fn toggle_track_mute(
    ev: On<Activate>,
    cells: Query<&TrackMuteCell>,
    mut mute: ResMut<JamMidiMute>,
) {
    let Ok(cell) = cells.get(ev.entity) else {
        return;
    };
    if let Some(m) = mute.0.get_mut(cell.0) {
        *m = !*m;
    }
}

/// Keeps each mute button's icon/background in sync with `JamMidiMute`.
pub fn update_track_mute_buttons(
    mute: Res<JamMidiMute>,
    mut icons: Query<(&TrackMuteIcon, &mut Text)>,
    mut cells: Query<(&TrackMuteCell, &mut BackgroundColor)>,
) {
    for (icon, mut text) in &mut icons {
        if let Some(&muted) = mute.0.get(icon.0) {
            *text = Text::new(if muted { MUTED_ICON } else { SOUND_ICON });
        }
    }
    for (cell, mut bg) in &mut cells {
        if let Some(&muted) = mute.0.get(cell.0) {
            bg.0 = if muted { MUTED_BG } else { UNMUTED_BG };
        }
    }
}

/// Applies `JamMidiMute` to each track's own sink volume — muted → silent,
/// unmuted → the configured music volume. Ordered `.after(gameplay::
/// lifecycle::apply_music_volume)` so a mid-song global-volume change
/// (which touches every `MusicPlayer` sink, these included, since
/// `MidiTrackPlayer` entities carry that tag too) can never un-mute a
/// muted track — this system always has the last word.
pub fn apply_midi_track_mute(
    mute: Res<JamMidiMute>,
    audio: Res<AudioSettings>,
    mut sinks: Query<(&MidiTrackPlayer, &mut AudioSink)>,
) {
    for (player, mut sink) in &mut sinks {
        let muted = mute.0.get(player.0).copied().unwrap_or(false);
        sink.set_volume(Volume::Linear(if muted { 0.0 } else { audio.music_volume }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jam_midi_mute_defaults_to_empty() {
        assert!(JamMidiMute::default().0.is_empty());
    }
}
