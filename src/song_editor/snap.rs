// SPDX-License-Identifier: MIT

//! Which within-beat tick positions a click on the note grid can land a new
//! note on — split out of `state.rs` purely to stay under its line budget
//! (`docs/physical_design_plan.md`), not because this is a separate feature
//! area; `EditorState::snap_mode` is still state.rs's own field.

use super::TICKS_PER_BEAT;

/// `TICKS_PER_BEAT` (12) is the lowest resolution divisible by both 4
/// (straight 16ths) and 3 (triplets), so every mode's positions below are
/// exact integer ticks — no rounding error the way a true triplet would
/// have had on the old 4-ticks-per-beat grid. See [`SnapMode::grid_points`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum SnapMode {
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
    pub(super) fn grid_points(self) -> &'static [usize] {
        match self {
            SnapMode::Sixteenth => &[0, 3, 6, 9],
            SnapMode::Shuffle => &[0, 8],
            SnapMode::Triplet => &[0, 4, 8],
        }
    }

    pub(super) fn label_key(self) -> &'static str {
        match self {
            SnapMode::Sixteenth => "editor-snap-mode-sixteenth",
            SnapMode::Shuffle => "editor-snap-mode-shuffle",
            SnapMode::Triplet => "editor-snap-mode-triplet",
        }
    }

    pub(super) fn next(self) -> SnapMode {
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
pub(super) fn snap_tick_in_beat(frac: f32, mode: SnapMode) -> usize {
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
    fn cycling_snap_mode_visits_all_three_and_wraps() {
        assert_eq!(SnapMode::Sixteenth.next(), SnapMode::Shuffle);
        assert_eq!(SnapMode::Shuffle.next(), SnapMode::Triplet);
        assert_eq!(SnapMode::Triplet.next(), SnapMode::Sixteenth);
    }
}
