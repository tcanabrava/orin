// SPDX-License-Identifier: MIT

//! The editor's view of [`harmonicon_core::pitch_map`].
//!
//! The resolution itself — which hole, breath and technique produces a
//! pitch, and which harp key fits a set of notes — now lives in core, where
//! gameplay and score importers can reach it too. What stays here is the
//! translation into the editor's own `Dir`/`Pitch`/`HarmonicaKind`
//! vocabulary, which exists because those are what the mod-panel buttons
//! produce and what a `GridNote` stores.
//!
//! Kept as a module rather than dissolved into its two callers so that MIDI
//! import (which wants every note to land *somewhere* — [`map_pitch`]) and
//! live recording (which wants to *discard* what the harp can't have made —
//! [`map_pitch_playable`]) still name the same pair of functions they always
//! did.

use super::state::{Dir, HarmonicaKind, Pitch};
use harmonicon_core::chart::Action;
use harmonicon_core::harmonica::Harmonica;
use harmonicon_core::pitch_map::{self, HarpKind, HoleAssignment, Technique};

/// The editor's `HarmonicaKind` as core's `HarpKind`.
pub(super) fn harp_kind(kind: HarmonicaKind) -> HarpKind {
    match kind {
        HarmonicaKind::Diatonic => HarpKind::Diatonic,
        HarmonicaKind::Chromatic => HarpKind::Chromatic,
    }
}

/// Core's resolution in the editor's own terms.
///
/// Note that core's `Technique::Overblow`/`Overdraw` arms are reachable now
/// in a way they weren't before: the editor's old resolver stopped at bends
/// and the chromatic slide, so an unreachable note fell through to the
/// nearest-note fallback instead. Both map straight onto the `Pitch`
/// variants the mod panel already had.
fn from_core(assignment: HoleAssignment) -> (u8, Dir, Pitch) {
    let dir = match assignment.action {
        Action::Blow => Dir::Blow,
        Action::Draw => Dir::Draw,
    };
    let pitch = match assignment.technique {
        Technique::Natural => Pitch::Normal,
        Technique::Bend(depth) => Pitch::Bend(depth),
        Technique::Overblow => Pitch::Overblow,
        Technique::Overdraw => Pitch::Overdraw,
        Technique::Slide => Pitch::Slide,
    };
    (assignment.hole, dir, pitch)
}

/// Resolves `target` onto `harp` only if it can genuinely produce it.
///
/// `kind` is accepted for call-site continuity but no longer consulted: core
/// reads the harp family off the [`Harmonica`] itself, which is the one
/// source that cannot disagree with the layout being searched.
pub(super) fn map_pitch_playable(
    target: u8,
    harp: &Harmonica,
    _kind: HarmonicaKind,
) -> Option<(u8, Dir, Pitch)> {
    pitch_map::map_pitch_playable(target, harp).map(from_core)
}

/// [`map_pitch_playable`] with core's nearest-natural-note fallback.
pub(super) fn map_pitch(target: u8, harp: &Harmonica, _kind: HarmonicaKind) -> (u8, Dir, Pitch) {
    from_core(pitch_map::map_pitch(target, harp))
}

/// The harp key needing the fewest bends, overblows and fallbacks.
pub(super) fn suggest_key(midi_keys: &[u8], kind: HarmonicaKind) -> &'static str {
    pitch_map::suggest_key(midi_keys, harp_kind(kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harmonicon_core::harmonica::{chromatic_harp, richter_harp};

    /// The translation, not the resolution — core owns and tests the latter.
    #[test]
    fn core_techniques_arrive_as_the_editors_own_pitch_variants() {
        for (technique, expected) in [
            (Technique::Natural, Pitch::Normal),
            (Technique::Bend(1.5), Pitch::Bend(1.5)),
            (Technique::Overblow, Pitch::Overblow),
            (Technique::Overdraw, Pitch::Overdraw),
            (Technique::Slide, Pitch::Slide),
        ] {
            let (_, _, pitch) = from_core(HoleAssignment {
                hole: 4,
                action: Action::Blow,
                technique,
            });
            assert_eq!(pitch, expected);
        }
    }

    #[test]
    fn both_breath_directions_survive_the_translation() {
        for (action, expected) in [(Action::Blow, Dir::Blow), (Action::Draw, Dir::Draw)] {
            let (_, dir, _) = from_core(HoleAssignment {
                hole: 1,
                action,
                technique: Technique::Natural,
            });
            assert_eq!(dir, expected);
        }
    }

    /// The two behaviours the editor's callers actually depend on: import
    /// always lands somewhere, recording rejects what the harp can't make.
    #[test]
    fn import_always_resolves_where_recording_refuses() {
        let harp = richter_harp("C");
        assert_eq!(map_pitch_playable(0, &harp, HarmonicaKind::Diatonic), None);
        let (hole, _, pitch) = map_pitch(0, &harp, HarmonicaKind::Diatonic);
        assert_eq!(pitch, Pitch::Normal);
        assert!((1..=10).contains(&hole));
    }

    #[test]
    fn a_chromatic_harp_still_resolves_its_slide() {
        let harp = chromatic_harp("C");
        let c4 = harmonicon_core::midi::note_to_midi("C4").unwrap() as u8;
        assert_eq!(
            map_pitch_playable(c4 + 1, &harp, HarmonicaKind::Chromatic),
            Some((1, Dir::Blow, Pitch::Slide))
        );
    }
}
