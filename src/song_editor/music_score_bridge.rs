// SPDX-License-Identifier: MIT

//! Wires the Song Editor's own note/playhead state into the shared
//! `music_score` overlay — the editor's sibling of `gameplay::
//! music_score_bridge`, translating this module's own vocabulary
//! (`EditorState::notes`, `playback::Playhead`) into the shared plugin's
//! (`MusicScoreNotes`/`MusicScorePlayhead`) instead of gameplay's.
//! Editor ticks are already tempo-independent multiples of a beat
//! (`TICKS_PER_BEAT`), so — unlike gameplay's own bridge — this needs no
//! tempo-map conversion at all, just a division.

use bevy::prelude::*;

use crate::music_score::{MusicScoreNotes, MusicScorePlayhead, NotationNote, split_at_bar_lines};

use super::TICKS_PER_BEAT;
use super::playback::{Playhead, build_harp, note_midi};
use super::state::EditorState;

/// Rebuilds [`MusicScoreNotes`] from `EditorState::notes` whenever the
/// editor state changes — same `resource_exists_and_changed::<EditorState>`
/// gate every other EditorState-derived rebuild in `song_editor::mod`'s own
/// system list uses. A note whose hole/technique the current harp can't
/// resolve (e.g. an Overblow on a hole that doesn't support it) has nothing
/// to draw and is skipped, same as gameplay's own bridge. `super::
/// BEATS_PER_BAR` (the editor has no editable time signature of its own,
/// see that constant's own definition) feeds `split_at_bar_lines` so a note
/// crossing a bar line becomes several tied segments instead of one
/// oversized notehead.
pub(super) fn sync_music_score(state: Res<EditorState>, mut notes: ResMut<MusicScoreNotes>) {
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
        .flat_map(|note| split_at_bar_lines(note, super::BEATS_PER_BAR as f64))
        .collect();
}

/// Keeps [`MusicScorePlayhead`] following the same tick position
/// `playback::update_playhead_view`'s moving line already derives from
/// [`Playhead`] — ordered `.after(playback::advance_playhead)` like that
/// system, so it reads the same frame's `elapsed`, not last frame's.
pub(super) fn sync_music_score_playhead(
    playhead: Res<Playhead>,
    mut score_playhead: ResMut<MusicScorePlayhead>,
) {
    if playhead.secs_per_tick <= 0.0 {
        return;
    }
    let cur_tick = playhead.elapsed / playhead.secs_per_tick;
    score_playhead.0 = (cur_tick / TICKS_PER_BEAT as f32) as f64;
}
