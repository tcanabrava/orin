// SPDX-License-Identifier: MIT

use bevy::{
    audio::{AudioSource, Volume},
    prelude::*,
};
use bevy_fluent::Localization;

use crate::{
    app::{AppState, GameplayMode, SelectedSong},
    localization::LocalizationExt,
    song::SongManifest,
};
use harmonicon_audio::AudioSettings;

use super::{
    GameplayClock, GameplayLogic, GameplayRoot, MidiTrackPlayer, MusicPlayer, MusicStarted, Paused,
};

#[derive(Component, Default, Clone)]
pub struct CountdownOverlay;

#[derive(Component)]
pub struct CountdownText;

pub fn spawn_countdown(commands: &mut Commands, loc: &Localization, harp_hint: Option<&str>) {
    // The full-screen overlay shell is static and font/handle-free, so it's a
    // `bsn!` scene. The countdown text children carry a custom `FontSource`,
    // which `bsn!` can't take directly in 0.19-rc.3, so they stay imperative.
    let overlay = commands
        .spawn_scene(bsn! {
            Node {
                position_type: {PositionType::Absolute},
                width: {Val::Percent(100.0)},
                height: {Val::Percent(100.0)},
                flex_direction: {FlexDirection::Column},
                align_items: {AlignItems::Center},
                justify_content: {JustifyContent::Center},
                row_gap: {Val::Px(12.0)},
            }
            BackgroundColor({Color::srgba(0.0, 0.0, 0.05, 0.55)})
            GlobalZIndex(100)
            CountdownOverlay
            GameplayRoot
        })
        .id();
    commands.entity(overlay).with_children(|ov| {
        ov.spawn((
            Text::new(String::from(loc.msg("gameplay-get-ready"))),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::srgba(0.85, 0.85, 1.0, 0.80)),
        ));
        // Which physical harp to grab (2D/3D pass this; jam shows it elsewhere).
        if let Some(hint) = harp_hint {
            ov.spawn((
                Text::new(hint.to_string()),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.80, 0.35)),
            ));
        }
        ov.spawn((
            Text::new("3"),
            TextFont {
                font_size: FontSize::Px(120.0),
                ..default()
            },
            TextColor(Color::WHITE),
            CountdownText,
        ));
    });
}

pub fn update_countdown(
    clock: Res<GameplayClock>,
    mut overlay: Query<&mut Visibility, With<CountdownOverlay>>,
    mut text: Query<(&mut Text, &mut TextFont), With<CountdownText>>,
    mut music_started: ResMut<MusicStarted>,
    selected: Res<SelectedSong>,
    manifests: Res<Assets<SongManifest>>,
    audio: Res<AudioSettings>,
    mode: Res<GameplayMode>,
    mut commands: Commands,
) {
    if clock.get() >= 0.0 {
        for mut vis in &mut overlay {
            *vis = Visibility::Hidden;
        }
        if !music_started.0 {
            music_started.0 = true;
            // `manifest.music` is `None` for a song with no `song/*.ogg`/
            // `*.wav` — play the chart silently rather than not starting at
            // all (the clock free-runs on frame delta instead of anchoring
            // to a sink; see `gameplay::should_anchor_to_sink`), *unless*
            // the song ships `midi_tracks` instead (see `song::
            // MidiTrackAudio`), in which case every track gets its own
            // sink, all spawned together this same frame so they start in
            // sync — muting one later is just zeroing that sink's volume
            // (`jam::midi_tracks::apply_midi_track_mute`), no re-mixing.
            if let Some(manifest) = manifests.get(&selected.0) {
                // Jam Session's own `restart_finished_jam_music` re-spawns
                // these entities once they despawn themselves, if Loop is on
                // at that moment — so they always start as plain one-shots
                // that self-clean; scored modes need the same self-cleaning
                // one-shot to move on to the results screen.
                let settings = if *mode == GameplayMode::JamSession {
                    PlaybackSettings::DESPAWN
                } else {
                    PlaybackSettings::ONCE
                };
                if let Some(music) = manifest.music.clone() {
                    commands.spawn((
                        AudioPlayer::<AudioSource>(music),
                        settings.with_volume(Volume::Linear(audio.music_volume)),
                        MusicPlayer,
                        GameplayRoot,
                    ));
                } else if let Some(tracks) = &manifest.midi_tracks {
                    for (index, track) in tracks.iter().enumerate() {
                        commands.spawn((
                            AudioPlayer::<AudioSource>(track.source.clone()),
                            settings.with_volume(Volume::Linear(audio.music_volume)),
                            MusicPlayer,
                            MidiTrackPlayer(index),
                            GameplayRoot,
                        ));
                    }
                }
            }
        }
        return;
    }

    for mut vis in &mut overlay {
        *vis = Visibility::Visible;
    }

    let remaining = -clock.get();
    let n = remaining.ceil() as u32;
    let frac = remaining.fract() as f32;
    let font_size = 80.0 + (1.0 - frac) * 80.0;

    for (mut t, mut font) in &mut text {
        t.0 = format!("{n}");
        font.font_size = FontSize::Px(font_size);
    }
}

pub struct CountdownPlugin;

impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_countdown
                .after(GameplayLogic)
                .run_if(in_state(AppState::Playing).and_then(|p: Res<Paused>| !p.0)),
        );
    }
}
