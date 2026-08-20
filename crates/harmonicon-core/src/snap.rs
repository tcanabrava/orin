// SPDX-License-Identifier: MIT

//! Which within-beat tick positions a click on the note grid can land a new
//! note on — split out of `state.rs` purely to stay under its line budget
//! (`docs/physical_design_plan.md`), not because this is a separate feature
//! area; `EditorState::snap_mode` is still state.rs's own field.

use crate::synth::TICKS_PER_BEAT;

/// `TICKS_PER_BEAT` (12) is the lowest resolution divisible by both 4
/// (straight 16ths) and 3 (triplets), so every mode's positions below are
/// exact integer ticks — no rounding error the way a true triplet would
/// have had on the old 4-ticks-per-beat grid. See [`SnapMode::grid_points`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SnapMode {
    /// Straight 16th notes — ticks 0, 3, 6, 9. What every click already
    /// snapped to before this mode existed (all 4 of `TICKS_PER_BEAT`'s old
    /// positions, just expressed at the new finer resolution).
    #[default]
    Sixteenth,
    /// Swung ("shuffle") 8th notes — a 2:1 long-short pair, ticks 0 and 8.
    /// This is the classic blues shuffle feel: play the first and third
    /// notes of an 8th-note triplet, skip the middle one.
    Shuffle,
    /// Straight 8th-note triplets — three equal subdivisions, ticks 0, 4, 8.
    Triplet,
}

impl SnapMode {
    /// The tick offsets (0..`TICKS_PER_BEAT`) this mode allows landing a new
    /// note on, within a single beat.
    pub fn grid_points(self) -> &'static [usize] {
        match self {
            SnapMode::Sixteenth => &[0, 3, 6, 9],
            SnapMode::Shuffle => &[0, 8],
            SnapMode::Triplet => &[0, 4, 8],
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            SnapMode::Sixteenth => "editor-snap-mode-sixteenth",
            SnapMode::Shuffle => "editor-snap-mode-shuffle",
            SnapMode::Triplet => "editor-snap-mode-triplet",
        }
    }

    pub fn next(self) -> SnapMode {
        match self {
            SnapMode::Sixteenth => SnapMode::Shuffle,
            SnapMode::Shuffle => SnapMode::Triplet,
            SnapMode::Triplet => SnapMode::Sixteenth,
        }
    }
}

/// Snaps a fractional position within a beat (`0.0..1.0`, e.g. a click's
/// normalized offset across a beat cell) to the nearest tick `mode` allows.
/// Pure so it's unit-testable without spinning up a grid click.
pub fn snap_tick_in_beat(frac: f32, mode: SnapMode) -> usize {
    let raw = frac.clamp(0.0, 0.999) * TICKS_PER_BEAT as f32;
    mode.grid_points()
        .iter()
        .copied()
        .min_by(|&a, &b| {
            (raw - a as f32)
                .abs()
                .partial_cmp(&(raw - b as f32).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0)
}

/// Snaps an *absolute* tick position (unlike [`snap_tick_in_beat`]'s
/// fractional position within a single beat) to the nearest tick `mode`
/// allows, across beat boundaries — used by drag-to-move/-resize so an
/// existing note snaps onto a shuffle/triplet position the same way a new
/// one can. `grid_points()` always includes 0, so the current beat's own
/// points plus the *next* beat's tick 0 are the only candidates that
/// matter.
pub fn snap_absolute_tick(tick: usize, mode: SnapMode) -> usize {
    let beat = tick / TICKS_PER_BEAT;
    let mut best = beat * TICKS_PER_BEAT;
    let mut best_dist = tick - best;
    for &p in mode.grid_points() {
        let candidate = beat * TICKS_PER_BEAT + p;
        let dist = tick.abs_diff(candidate);
        if dist < best_dist {
            best = candidate;
            best_dist = dist;
        }
    }
    let next_beat_start = (beat + 1) * TICKS_PER_BEAT;
    if next_beat_start - tick < best_dist {
        best = next_beat_start;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteenth_mode_reproduces_the_old_any_tick_grid() {
        assert_eq!(snap_tick_in_beat(0.0, SnapMode::Sixteenth), 0);
        assert_eq!(snap_tick_in_beat(0.26, SnapMode::Sixteenth), 3);
        assert_eq!(snap_tick_in_beat(0.5, SnapMode::Sixteenth), 6);
        assert_eq!(snap_tick_in_beat(0.76, SnapMode::Sixteenth), 9);
    }

    #[test]
    fn shuffle_mode_only_lands_on_the_long_short_pair() {
        // grid_points = [0, 8]; the midpoint (raw tick 4, frac 1/3) is where
        // the nearest point flips from 0 to 8.
        assert_eq!(snap_tick_in_beat(0.0, SnapMode::Shuffle), 0);
        assert_eq!(snap_tick_in_beat(0.3, SnapMode::Shuffle), 0);
        assert_eq!(snap_tick_in_beat(0.4, SnapMode::Shuffle), 8);
        assert_eq!(snap_tick_in_beat(0.7, SnapMode::Shuffle), 8);
        assert_eq!(snap_tick_in_beat(0.99, SnapMode::Shuffle), 8);
    }

    #[test]
    fn triplet_mode_lands_on_three_equal_subdivisions() {
        // grid_points = [0, 4, 8]; midpoints at raw ticks 2 and 6.
        assert_eq!(snap_tick_in_beat(0.0, SnapMode::Triplet), 0);
        assert_eq!(snap_tick_in_beat(0.3, SnapMode::Triplet), 4);
        assert_eq!(snap_tick_in_beat(0.5, SnapMode::Triplet), 4);
        assert_eq!(snap_tick_in_beat(0.7, SnapMode::Triplet), 8);
    }

    #[test]
    fn snap_absolute_tick_snaps_within_the_current_beat() {
        // Sixteenth: [0, 3, 6, 9] within beat 0 (ticks 0..12).
        assert_eq!(snap_absolute_tick(0, SnapMode::Sixteenth), 0);
        assert_eq!(snap_absolute_tick(4, SnapMode::Sixteenth), 3);
        assert_eq!(snap_absolute_tick(5, SnapMode::Sixteenth), 6);
        assert_eq!(snap_absolute_tick(10, SnapMode::Sixteenth), 9);
    }

    #[test]
    fn snap_absolute_tick_can_wrap_forward_into_the_next_beat() {
        // Tick 11 (beat 0) is one away from beat 1's own tick 0 (12), but
        // two away from beat 0's own last Sixteenth point (9) -> snaps
        // forward across the beat boundary rather than staying in beat 0.
        assert_eq!(snap_absolute_tick(11, SnapMode::Sixteenth), 12);
        // Sanity: one tick earlier still resolves within beat 0.
        assert_eq!(snap_absolute_tick(10, SnapMode::Sixteenth), 9);
    }

    #[test]
    fn snap_absolute_tick_never_snaps_backward_past_the_beat_it_started_in() {
        // A tick just after a beat boundary is always closer to that beat's
        // own 0 than to the previous beat's last point, for every mode —
        // e.g. tick 13 (beat 1, one tick in) must resolve to 12, not to
        // beat 0's own last Shuffle point (8).
        assert_eq!(snap_absolute_tick(13, SnapMode::Shuffle), 12);
        assert_eq!(snap_absolute_tick(13, SnapMode::Triplet), 12);
    }

    #[test]
    fn snap_absolute_tick_works_in_a_later_beat() {
        // Beat 2 starts at tick 24; Triplet's points there are 24, 28, 32.
        assert_eq!(snap_absolute_tick(29, SnapMode::Triplet), 28);
        assert_eq!(snap_absolute_tick(31, SnapMode::Triplet), 32);
    }

    #[test]
    fn cycling_snap_mode_visits_all_three_and_wraps() {
        assert_eq!(SnapMode::Sixteenth.next(), SnapMode::Shuffle);
        assert_eq!(SnapMode::Shuffle.next(), SnapMode::Triplet);
        assert_eq!(SnapMode::Triplet.next(), SnapMode::Sixteenth);
    }
}
