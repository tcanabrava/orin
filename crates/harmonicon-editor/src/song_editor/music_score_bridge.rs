// SPDX-License-Identifier: MIT

//! Wires the Song Editor's own note/playhead state into the shared
//! `music_score` overlay — the editor's sibling of `gameplay::
//! music_score_bridge`, translating `EditorState::notes`/`playback::
//! Playhead` into `MusicScoreNotes`/`MusicScorePlayhead`. Editor ticks are
//! already tempo-independent multiples of a beat (`TICKS_PER_BEAT`), so
//! unlike gameplay's bridge this needs no tempo-map conversion, just a
//! division.

use bevy::prelude::*;

use harmonicon_ui::music_score::MusicScoreMeter;
use harmonicon_ui::music_score::{
    MusicScoreNotes, MusicScorePlayhead, NotationNote, parse_time_signature, split_at_bar_lines,
};

use super::TICKS_PER_BEAT;
use super::playback::{Playhead, build_harp, note_midi};
use super::state::EditorState;

/// Rebuilds [`MusicScoreNotes`] from `EditorState::notes` whenever the
/// editor state changes — same `resource_exists_and_changed::<EditorState>`
/// gate every other EditorState-derived rebuild in `song_editor::mod` uses.
/// A note whose hole/technique the current harp can't resolve is skipped,
/// same as gameplay's bridge. `super::BEATS_PER_BAR` feeds
/// `split_at_bar_lines` so a note crossing a bar line becomes tied segments
/// instead of one oversized notehead.
pub(super) fn sync_music_score(
    state: Res<EditorState>,
    mut notes: ResMut<MusicScoreNotes>,
    mut meter: ResMut<MusicScoreMeter>,
) {
    let editor_meter = parse_time_signature(&state.time_signature);
    if *meter != editor_meter {
        *meter = editor_meter;
    }
    let harp = build_harp(&state.key, state.harmonica_kind);
    notes.0 = state
        .notes
        .iter()
        .filter_map(|n| {
            let midi = note_midi(n, &harp)?;
            Some(NotationNote {
                start_beat: n.tick as f64 / TICKS_PER_BEAT as f64,
                duration_beats: n.len.max(1) as f64 / TICKS_PER_BEAT as f64,
                midi,
                tied_from_previous: false,
            })
        })
        .flat_map(|note| split_at_bar_lines(note, editor_meter.beats_per_bar()))
        .collect();
}

/// Keeps [`MusicScorePlayhead`] following the same tick position
/// `playback::update_playhead_view`'s moving line derives from [`Playhead`]
/// while a take is playing (or paused mid-take) — ordered `.after(playback
/// ::advance_playhead)` so it reads the same frame's `elapsed`. Otherwise
/// (just editing, nothing running) it follows `EditorState::scroll_beat`
/// instead, so a note far from the song's start can still scroll into the
/// panel's fixed-width visible window rather than the score staying pinned
/// at beat 0.
pub(super) fn sync_music_score_playhead(
    playhead: Res<Playhead>,
    state: Res<EditorState>,
    mut score_playhead: ResMut<MusicScorePlayhead>,
) {
    if playhead.playing && playhead.secs_per_tick > 0.0 {
        let cur_tick = playhead.elapsed / playhead.secs_per_tick;
        score_playhead.0 = (cur_tick / TICKS_PER_BEAT as f32) as f64;
    } else {
        score_playhead.0 = state.scroll_beat as f64;
    }
}
