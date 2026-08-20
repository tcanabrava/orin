// SPDX-License-Identifier: MIT

//! Wires gameplay's own note/clock state into the shared `music_score`
//! overlay — Play 2D and Play 3D only (Jam Session has no [`SongNotes`] to
//! draw from, same reason it's excluded from scoring). Kept as its own
//! small plugin, separate from `notes.rs` (pure data/logic, no systems) and
//! `song_progress_overlay.rs` (a different overlay), since this is purely
//! translation: `SongNotes`/`GameplayClock` (gameplay's own vocabulary) into
//! [`MusicScoreNotes`]/[`MusicScorePlayhead`] (the shared plugin's).

use bevy::prelude::*;

use harmonicon_app::app::{AppState, GameplayMode, SelectedSong};
use harmonicon_core::chart::seconds_to_tick;
use harmonicon_song::song::SongManifest;
use harmonicon_ui::music_score::{MusicScoreNotes, MusicScorePlayhead};

use super::{GameplayClock, GameplayLogic, SongNotes, notes_to_notation};

pub struct MusicScoreBridgePlugin;

impl Plugin for MusicScoreBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_music_score_notes.run_if(resource_changed::<SongNotes>),
                update_music_score_playhead.after(GameplayLogic),
            )
                .run_if(in_state(AppState::Playing).and_then(playing_2d_or_3d)),
        );
    }
}

fn playing_2d_or_3d(mode: Res<GameplayMode>) -> bool {
    matches!(*mode, GameplayMode::Play2D | GameplayMode::Play3D)
}

/// Rebuilds [`MusicScoreNotes`] whenever [`SongNotes`] changes — song setup,
/// and any adaptive-difficulty resync mid-song (`gameplay_2d`/`gameplay_3d`'s
/// `resync_notes_on_adaptive_change`), so the staff stays in step with
/// whichever notes are actually live.
fn sync_music_score_notes(
    song_notes: Res<SongNotes>,
    selected: Res<SelectedSong>,
    manifests: Res<Assets<SongManifest>>,
    mut score_notes: ResMut<MusicScoreNotes>,
) {
    let Some(manifest) = manifests.get(&selected.0) else {
        return;
    };
    let timing = &manifest.chart.timing;
    let beats_per_bar = manifest
        .chart
        .song
        .time_signature
        .as_deref()
        .unwrap_or("4/4")
        .split('/')
        .next()
        .and_then(|n| n.parse::<f64>().ok())
        .unwrap_or(4.0);
    score_notes.0 = notes_to_notation(
        &song_notes.notes,
        timing.resolution,
        &timing.tempo_map,
        beats_per_bar,
    );
}

/// Keeps [`MusicScorePlayhead`] following the same [`GameplayClock`] every
/// other clock-reading system uses — ordered `.after(GameplayLogic)` per
/// that invariant (see `gameplay/plugin.rs`'s own doc comment on it).
fn update_music_score_playhead(
    clock: Res<GameplayClock>,
    selected: Res<SelectedSong>,
    manifests: Res<Assets<SongManifest>>,
    mut playhead: ResMut<MusicScorePlayhead>,
) {
    let Some(manifest) = manifests.get(&selected.0) else {
        return;
    };
    let timing = &manifest.chart.timing;
    let tick = seconds_to_tick(clock.get().max(0.0), timing.resolution, &timing.tempo_map);
    playhead.0 = tick as f64 / timing.resolution.max(1) as f64;
}
