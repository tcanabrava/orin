// SPDX-License-Identifier: MIT

//! Surfaces the outcome of a Save/Load (`harpchart.rs`/`lesson_form.rs`) in
//! the status bar, visible in a non-terminal launch of a packaged build —
//! the same "one more priority tier" shape `panel::update_status_bar`
//! already uses for drag/record/practice messages. Every outcome is also
//! still logged via `info!`/`warn!`, structured and filterable for a
//! developer running from a terminal.

use bevy::prelude::*;

use harmonicon_platform::localization::LocalizedStr;

/// How long a save/load outcome stays in the status bar before falling
/// back to whatever it would otherwise show (drag/record/practice) — long
/// enough to actually read, short enough not to linger indefinitely over
/// unrelated later status messages.
const DISPLAY_SECS: f32 = 4.0;

/// The most recent save/load outcome, if still within its display window.
/// `set` (called from `harpchart`/`lesson_form`'s Save/Load systems)
/// always (re)starts the countdown, so a second save shortly after the
/// first gets its own full display window instead of inheriting the
/// first's leftover time.
#[derive(Resource, Default)]
pub(super) struct SaveFeedback {
    message: Option<LocalizedStr>,
    remaining_secs: f32,
}

impl SaveFeedback {
    pub(super) fn set(&mut self, message: LocalizedStr) {
        self.message = Some(message);
        self.remaining_secs = DISPLAY_SECS;
    }

    pub(super) fn current(&self) -> Option<&LocalizedStr> {
        self.message.as_ref()
    }
}

/// Pure step: `remaining` minus `dt`, or `None` once it reaches zero — same
/// shape as `settings::tick_debounce`, kept separate from the ECS system so
/// it's unit-testable without a `Time` resource.
fn tick(remaining_secs: f32, dt: f32) -> Option<f32> {
    let next = remaining_secs - dt;
    (next > 0.0).then_some(next)
}

/// Counts an active [`SaveFeedback`] message down, clearing it once its
/// display window elapses.
pub(super) fn tick_save_feedback(time: Res<Time>, mut feedback: ResMut<SaveFeedback>) {
    if feedback.message.is_none() {
        return;
    }
    match tick(feedback.remaining_secs, time.delta_secs()) {
        Some(remaining) => feedback.remaining_secs = remaining,
        None => feedback.message = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fluent::prelude::Localization;
    use harmonicon_platform::localization::LocalizationExt;

    fn msg(text: &str) -> LocalizedStr {
        // `Localization::default()` has no bundle loaded, so `loc.msg(key)`
        // falls back to the key itself — good enough to get a `LocalizedStr`
        // to test with, since these tests only care about the timer.
        Localization::default().msg(text)
    }

    #[test]
    fn a_fresh_message_starts_at_full_display_time() {
        let mut feedback = SaveFeedback::default();
        feedback.set(msg("hello"));
        assert_eq!(feedback.current(), Some(&msg("hello")));
        assert_eq!(feedback.remaining_secs, DISPLAY_SECS);
    }

    #[test]
    fn setting_a_new_message_restarts_the_countdown() {
        let mut feedback = SaveFeedback::default();
        feedback.set(msg("first"));
        feedback.remaining_secs = 0.5;
        feedback.set(msg("second"));
        assert_eq!(feedback.current(), Some(&msg("second")));
        assert_eq!(feedback.remaining_secs, DISPLAY_SECS);
    }

    // ── tick ──────────────────────────────────────────────────────────────

    #[test]
    fn tick_counts_down_without_expiring_before_it_elapses() {
        assert_eq!(tick(2.0, 0.5), Some(1.5));
    }

    #[test]
    fn tick_expires_once_it_reaches_zero() {
        assert_eq!(tick(0.5, 0.5), None);
    }

    #[test]
    fn tick_expires_past_zero_too() {
        assert_eq!(tick(0.2, 1.0), None);
    }
}
