// SPDX-License-Identifier: MIT

//! Turning any [`ScoreFile`] track into a playable [`HarpChart`].
//!
//! This is where a file that knows nothing about harmonicas becomes one:
//! every pitch is resolved onto a hole, breath and technique by
//! `harmonicon_core::pitch_map`, the same resolver the Song Editor's MIDI
//! import and live recording already use. One mapper, so an imported chart
//! can't disagree with an authored one about what a harp can play.
//!
//! **A converted chart is only as playable as the part it came from.** A
//! guitar line on a harmonica is mostly notes out of reach; `pick_harmonica_
//! track` exists to steer callers away from that, and [`ConversionReport`]
//! says plainly how much of what came out is actually reachable, so a caller
//! can refuse rather than hand the player an unplayable chart.

use harmonicon_core::chart::{
    Difficulty, HarpChart, Metadata, Modifier, NoteEvent, Scoring, Song, TempoPoint, Timing,
    TrackItem,
};
use harmonicon_core::harmonica::Harmonica;
use harmonicon_core::midi::midi_to_note;
use harmonicon_core::pitch_map::{Technique, map_pitch_playable};

use crate::{ScoreError, ScoreFile};

/// How well a track survived being put on a harmonica.
///
/// Reported rather than silently absorbed: converting a part written for
/// another instrument routinely loses notes, and the honest answer to
/// "this file has no harmonica in it" is to say so, not to emit a chart
/// nobody can play.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ConversionReport {
    pub total: usize,
    /// Notes landing on a plain blow or draw reed.
    pub natural: usize,
    pub bends: usize,
    pub overblows: usize,
    /// Notes the harmonica cannot produce at all. These are dropped.
    pub unreachable: usize,
}

impl ConversionReport {
    /// Fraction of the source track that made it onto the harp.
    pub fn reachable_fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.total - self.unreachable) as f32 / self.total as f32
    }

    /// Whether this is worth offering to a player at all.
    ///
    /// The threshold is a judgement, not a measurement: below it the result
    /// reads as a broken chart rather than a hard one. A caller wanting a
    /// different line can use [`Self::reachable_fraction`] directly.
    pub fn is_worth_playing(&self) -> bool {
        self.total > 0 && self.reachable_fraction() >= 0.8
    }
}

/// Ticks per beat the generated chart uses.
///
/// Matches `harmonicon_core::synth::TICKS_PER_BEAT` — 12, divisible by both
/// 4 and 3 so straight sixteenths and triplets are both representable. A
/// chart carries its own `timing.resolution`, so this only has to be
/// self-consistent, but agreeing with the editor's grid means an imported
/// chart opens there without rescaling.
pub const TICKS_PER_BEAT: u32 = harmonicon_core::synth::TICKS_PER_BEAT as u32;

/// Converts one track onto `harp`.
pub fn to_chart(
    score: &dyn ScoreFile,
    track: usize,
    harp: &Harmonica,
    artist: &str,
) -> Result<(HarpChart, ConversionReport), ScoreError> {
    let notes = score.notes(track)?;
    let tempo = score.tempo_bpm().max(1.0);
    let (numerator, denominator) = score.time_signature();

    let mut report = ConversionReport::default();
    let mut items = Vec::new();

    for note in &notes {
        report.total += 1;
        // The strict resolver, not the always-resolves one: an importer
        // wanting every note to land *somewhere* is right for authoring,
        // where a human then fixes it, and wrong here, where the nearest
        // playable note would silently rewrite the tune.
        let Some(assignment) = map_pitch_playable(note.midi, harp) else {
            report.unreachable += 1;
            continue;
        };

        let modifiers = match assignment.technique {
            Technique::Natural => {
                report.natural += 1;
                Vec::new()
            }
            Technique::Bend(depth) => {
                report.bends += 1;
                // Charts store a bend as a negative (downward) offset.
                vec![Modifier::Bend {
                    semitones: -depth,
                    intensity: None,
                }]
            }
            Technique::Overblow => {
                report.overblows += 1;
                vec![Modifier::Overblow]
            }
            Technique::Overdraw => {
                report.overblows += 1;
                vec![Modifier::Overdraw]
            }
            Technique::Slide => {
                report.natural += 1;
                vec![Modifier::Slide]
            }
        };

        items.push(TrackItem {
            id: None,
            time: Some(note.start_secs),
            tick: None,
            duration: note.duration_secs,
            phrase: None,
            groove: None,
            play_mode: None,
            call: false,
            events: vec![NoteEvent {
                hole: assignment.hole,
                action: assignment.action,
                // The resulting pitch, written out. An overblow's or a
                // bend's sounding note is not derivable from its modifier
                // alone — the chart format expects it stated here.
                note: Some(midi_to_note(note.midi as i32)),
                modifiers: (!modifiers.is_empty()).then_some(modifiers),
            }],
        });
    }

    let chart = HarpChart {
        metadata: Some(Metadata {
            format_version: Some(harmonicon_core::chart::CURRENT_FORMAT_VERSION.to_string()),
            author: None,
            source: Some(format!("imported from {}", score.format().label())),
            license: None,
            description: None,
        }),
        song: Song {
            title: score.title().unwrap_or("Imported").to_string(),
            artist: artist.to_string(),
            tempo_bpm: tempo,
            key: harmonicon_core::harmonica::detected_harp_key(harp)
                .unwrap_or_else(|| "C".to_string()),
            // Imported material has no difficulty rating of its own, and
            // guessing one from note density would be a worse lie than a
            // neutral default the author can change.
            difficulty: Difficulty::Intermediate,
            time_signature: Some(format!("{numerator}/{denominator}")),
            feel: None,
        },
        timing: Timing {
            resolution: TICKS_PER_BEAT,
            // One point: every note already carries absolute seconds, so
            // the map is metadata for the metronome rather than the
            // timebase. Carrying a source file's full tempo automation
            // through is worth doing later; it changes nothing about when
            // notes land.
            tempo_map: vec![TempoPoint {
                tick: 0,
                bpm: tempo,
            }],
            time_signature_map: None,
        },
        harmonica: harp.clone(),
        track: items,
        loop_section: None,
        // The same windows every bundled chart uses.
        scoring: Scoring {
            perfect_window_ms: 50,
            good_window_ms: 100,
            miss_window_ms: 130,
            combo: None,
            style_bonus: None,
        },
    };

    Ok((chart, report))
}

#[cfg(test)]
mod tests;
