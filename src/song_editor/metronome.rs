// SPDX-License-Identifier: MIT

//! A click track for Record/Play/Practice, reusing `gameplay::
//! metronome_overlay`'s pure tick math, shared click-audio resources
//! (`MetronomeSounds`), and the same `MetronomeTempo`/`MetronomeFeel`/
//! `MetronomeMuted` globals gameplay and the Bending Trainer already use —
//! so a player's mute preference carries over into the editor instead of
//! resetting. Not gameplay's own click-driving systems though, which are
//! tied to `GameplayClock`: the editor has its own clock
//! (`playback::Playhead`), so [`click_metronome`] reads from that,
//! sharing only the pure click-selection logic
//! (`metronome_overlay::play_click_if_due`).
//!
//! Also owns the Record transport's count-in: pressing Play to start a
//! *fresh* take (not resuming a paused one) doesn't start capturing
//! immediately — [`CountIn`] ticks off one bar of clicks first, then
//! [`finish_count_in`] hands off to `record::start_record` for real, so
//! nothing is lost or misjudged by counting in first.

use bevy::audio::AudioSource;
use bevy::prelude::*;

use crate::audio_system::AudioSettings;
use crate::audio_system::pitch_detect::PitchRange;
use crate::gameplay::metronome_overlay::{
    MetronomeFeel, MetronomeMuted, MetronomeSounds, MetronomeTempo, play_click_if_due,
};

use super::playback::{EditorAudio, PendingMusicSeek, Playhead, secs_per_tick};
use super::record::{RecordState, start_record};
use super::state::EditorState;
use super::{BEATS_PER_BAR, TICKS_PER_BEAT};

/// The last click tick played this editing session — the editor's own
/// counterpart to `metronome_overlay::LastClickedTick`, kept separate so
/// switching between gameplay and the editor can't leave a stale tick
/// index behind that suppresses (or double-fires) the first click in
/// whichever context comes next.
#[derive(Resource, Default)]
pub(super) struct EditorLastClickedTick(Option<i64>);

/// A pending count-in before a fresh Record take actually starts
/// capturing — one bar of metronome clicks, giving the player a beat to
/// prepare, same as any DAW's count-in. Counts down in real (wall-clock)
/// time via `Time::delta`, not `Playhead::elapsed`: the take and its
/// clock haven't started yet, so there's nothing for `Playhead` to count.
#[derive(Resource, Default)]
pub(super) struct CountIn {
    total_secs: f32,
    remaining_secs: f32,
    active: bool,
}

impl CountIn {
    pub(super) fn active(&self) -> bool {
        self.active
    }

    /// Seconds left, for the status bar — `None` whenever there's nothing
    /// to show (mirrors `active`, but as the value a caller actually wants
    /// to display rather than a bare bool).
    pub(super) fn remaining_secs_display(&self) -> Option<f32> {
        self.active.then_some(self.remaining_secs)
    }

    /// Seconds already elapsed since the count-in started — what
    /// `play_click_if_due` needs as its "clock" to click on beat.
    fn elapsed_secs(&self) -> f32 {
        self.total_secs - self.remaining_secs
    }

    fn start(&mut self, total_secs: f32) {
        self.total_secs = total_secs;
        self.remaining_secs = total_secs;
        self.active = true;
    }

    pub(super) fn stop(&mut self) {
        self.active = false;
    }
}

/// Seconds of count-in for one bar at `bpm` — `BEATS_PER_BAR` beats,
/// converted the same way any other beat-to-seconds figure in this crate
/// is.
pub(super) fn count_in_secs(bpm: f32) -> f32 {
    BEATS_PER_BAR as f32 * 60.0 / bpm.max(1.0)
}

/// `EditorState::tempo`'s flat nominal BPM, recovered from `playback::
/// secs_per_tick` rather than re-parsing `state.tempo` directly, so the
/// two can't drift onto different fallback/clamp behaviour.
pub(super) fn tempo_bpm(state: &EditorState) -> f32 {
    60.0 / (secs_per_tick(state) * TICKS_PER_BEAT as f32)
}

/// Starts a count-in for a fresh Record take — called by the Record Play
/// button instead of `record::start_record` directly when there's no take
/// already in flight to resume. See [`CountIn`]'s own doc comment.
pub(super) fn begin_count_in(state: &EditorState, count_in: &mut CountIn) {
    count_in.start(count_in_secs(tempo_bpm(state)));
}

/// Keeps `MetronomeTempo` in step with the chart currently being edited —
/// the editor's own counterpart to
/// `metronome_overlay::set_tempo_from_song`, run continuously (rather than
/// once `OnEnter`) since the tempo field is itself live-editable. Only one
/// of gameplay/the Bending Trainer/the editor is ever active at a time
/// (different `AppState`s), so this can't fight the others over the same
/// shared resource — whichever context is entered next reseeds it.
pub(super) fn sync_tempo(state: Res<EditorState>, mut tempo: ResMut<MetronomeTempo>) {
    tempo.bpm = tempo_bpm(&state);
    tempo.beats_per_bar = BEATS_PER_BAR;
}

/// Plays the metronome clicks for the editor's own clock (`Playhead`)
/// while it's actually running — Record, Play, and Practice all drive the
/// same `Playhead`, so gating on it covers whichever is active without
/// needing to know which. Silent during a count-in ([`tick_count_in`]
/// clicks instead, against its own clock) and while nothing is playing.
pub(super) fn click_metronome(
    playhead: Res<Playhead>,
    count_in: Res<CountIn>,
    tempo: Res<MetronomeTempo>,
    muted: Res<MetronomeMuted>,
    feel: Res<MetronomeFeel>,
    sounds: Res<MetronomeSounds>,
    audio: Res<AudioSettings>,
    mut last: ResMut<EditorLastClickedTick>,
    mut commands: Commands,
) {
    if count_in.active() || !playhead.playing || playhead.paused {
        return;
    }
    play_click_if_due(
        playhead.elapsed as f64,
        &tempo,
        *feel,
        muted.0,
        &sounds,
        &audio,
        &mut last.0,
        &mut commands,
    );
}

/// Ticks a pending count-in: clicks against its own elapsed-since-start
/// clock (separate from `Playhead`, which isn't running yet — see
/// [`CountIn`]'s doc comment) and counts `remaining_secs` down by the
/// frame delta. Split from [`finish_count_in`] (which reacts once it
/// reaches zero) purely to stay under a single Bevy system's parameter
/// limit — the two always run back to back.
pub(super) fn tick_count_in(
    time: Res<Time>,
    mut count_in: ResMut<CountIn>,
    mut last: ResMut<EditorLastClickedTick>,
    tempo: Res<MetronomeTempo>,
    muted: Res<MetronomeMuted>,
    feel: Res<MetronomeFeel>,
    sounds: Res<MetronomeSounds>,
    audio: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !count_in.active() {
        return;
    }
    play_click_if_due(
        count_in.elapsed_secs() as f64,
        &tempo,
        *feel,
        muted.0,
        &sounds,
        &audio,
        &mut last.0,
        &mut commands,
    );
    count_in.remaining_secs -= time.delta_secs();
}

/// Hands off to `record::start_record` the instant [`tick_count_in`]'s
/// countdown reaches zero — see [`tick_count_in`]'s doc comment for why
/// this is a separate system rather than one.
#[allow(clippy::too_many_arguments)]
pub(super) fn finish_count_in(
    mut count_in: ResMut<CountIn>,
    mut last: ResMut<EditorLastClickedTick>,
    state: Res<EditorState>,
    mut sources: ResMut<Assets<AudioSource>>,
    settings: Res<AudioSettings>,
    playing: Query<Entity, With<EditorAudio>>,
    mut record: ResMut<RecordState>,
    mut playhead: ResMut<Playhead>,
    mut pitch_range: ResMut<PitchRange>,
    mut music_seek: ResMut<PendingMusicSeek>,
    mut commands: Commands,
) {
    if !count_in.active() || count_in.remaining_secs > 0.0 {
        return;
    }
    count_in.stop();
    last.0 = None;
    start_record(
        &state,
        &mut sources,
        &settings,
        &playing,
        &mut record,
        &mut playhead,
        &mut pitch_range,
        &mut music_seek,
        &mut commands,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CountIn ───────────────────────────────────────────────────────────

    #[test]
    fn count_in_starts_inactive() {
        let count_in = CountIn::default();
        assert!(!count_in.active());
        assert_eq!(count_in.remaining_secs_display(), None);
    }

    #[test]
    fn starting_activates_and_seeds_remaining_time() {
        let mut count_in = CountIn::default();
        count_in.start(2.0);
        assert!(count_in.active());
        assert_eq!(count_in.remaining_secs_display(), Some(2.0));
        assert_eq!(count_in.elapsed_secs(), 0.0);
    }

    #[test]
    fn elapsed_grows_as_remaining_shrinks() {
        let mut count_in = CountIn::default();
        count_in.start(2.0);
        count_in.remaining_secs = 0.5;
        assert_eq!(count_in.elapsed_secs(), 1.5);
    }

    #[test]
    fn stopping_deactivates() {
        let mut count_in = CountIn::default();
        count_in.start(2.0);
        count_in.stop();
        assert!(!count_in.active());
        assert_eq!(count_in.remaining_secs_display(), None);
    }

    // ── count_in_secs ─────────────────────────────────────────────────────

    #[test]
    fn count_in_secs_is_one_bar_at_the_given_bpm() {
        // 4 beats at 120bpm (0.5s/beat) = 2.0s.
        assert_eq!(count_in_secs(120.0), 2.0);
    }

    #[test]
    fn count_in_secs_scales_inversely_with_bpm() {
        assert_eq!(count_in_secs(60.0), 4.0);
        assert_eq!(count_in_secs(240.0), 1.0);
    }

    #[test]
    fn count_in_secs_clamps_a_nonpositive_bpm() {
        // Falls back to treating bpm as 1, rather than dividing by zero or
        // going negative.
        assert_eq!(count_in_secs(0.0), count_in_secs(1.0));
        assert_eq!(count_in_secs(-10.0), count_in_secs(1.0));
    }

    // ── tempo_bpm ─────────────────────────────────────────────────────────

    #[test]
    fn tempo_bpm_reads_the_editors_own_tempo_field() {
        let state = EditorState {
            tempo: "90".into(),
            ..Default::default()
        };
        assert!((tempo_bpm(&state) - 90.0).abs() < 1e-3);
    }

    #[test]
    fn tempo_bpm_falls_back_to_120_for_an_unparseable_tempo() {
        let state = EditorState {
            tempo: "not a number".into(),
            ..Default::default()
        };
        assert!((tempo_bpm(&state) - 120.0).abs() < 1e-3);
    }
}
