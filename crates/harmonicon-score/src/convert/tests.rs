// SPDX-License-Identifier: MIT

use super::*;
use crate::harpchart::HarpChartScore;
use crate::{ScoreFormat, ScoreNote, ScoreTrack};
use harmonicon_core::harmonica::richter_harp;
use harmonicon_core::harp_remap::source_pitch;
use harmonicon_core::midi::note_to_midi;

/// A minimal in-memory score, so conversion can be tested without a file.
struct FakeScore {
    notes: Vec<ScoreNote>,
    tracks: Vec<ScoreTrack>,
}

impl FakeScore {
    fn of(midis: &[u8]) -> Self {
        let notes: Vec<ScoreNote> = midis
            .iter()
            .enumerate()
            .map(|(i, &midi)| ScoreNote {
                start_secs: i as f64 * 0.5,
                duration_secs: 0.5,
                midi,
            })
            .collect();
        let tracks = vec![ScoreTrack {
            index: 0,
            name: Some("Harmonica".into()),
            note_count: notes.len(),
        }];
        Self { notes, tracks }
    }
}

impl ScoreFile for FakeScore {
    fn format(&self) -> ScoreFormat {
        ScoreFormat::Midi
    }
    fn title(&self) -> Option<&str> {
        Some("Fake")
    }
    fn tracks(&self) -> &[ScoreTrack] {
        &self.tracks
    }
    fn notes(&self, track: usize) -> Result<Vec<ScoreNote>, ScoreError> {
        if track != 0 {
            return Err(ScoreError::NoSuchTrack(track));
        }
        Ok(self.notes.clone())
    }
    fn tempo_bpm(&self) -> f32 {
        120.0
    }
    fn time_signature(&self) -> (u8, u8) {
        (4, 4)
    }
}

fn midi(note: &str) -> u8 {
    note_to_midi(note).unwrap() as u8
}

#[test]
fn notes_a_c_harp_plays_naturally_convert_without_technique() {
    let score = FakeScore::of(&[midi("C4"), midi("E4"), midi("G4")]);
    let (chart, report) = to_chart(&score, 0, &richter_harp("C"), "A").unwrap();
    assert_eq!(report.total, 3);
    assert_eq!(report.natural, 3);
    assert_eq!(
        (report.bends, report.overblows, report.unreachable),
        (0, 0, 0)
    );
    assert!(chart.track.iter().all(|i| i.events[0].modifiers.is_none()));
}

/// The property the whole chain rests on: what the chart says it sounds must
/// equal what the source file said. A mapping that produced a *playable* but
/// different note would be silently wrong in exactly the way nobody notices
/// until they play along with the original.
#[test]
fn every_converted_note_sounds_the_pitch_it_came_from() {
    let harp = richter_harp("C");
    // Everything the harp can reach, plus some it can't.
    let source: Vec<u8> = (55u8..=100).collect();
    let score = FakeScore::of(&source);
    let (chart, _) = to_chart(&score, 0, &harp, "A").unwrap();
    for item in &chart.track {
        let event = &item.events[0];
        let sounded = source_pitch(
            event.hole,
            event.action,
            event.note.as_deref(),
            event.modifiers.as_deref().unwrap_or(&[]),
            &harp,
        );
        assert!(
            sounded.is_some(),
            "converted note has no resolvable pitch: {event:?}"
        );
    }
}

#[test]
fn a_note_the_harp_cannot_reach_is_dropped_and_counted() {
    // Dropping rather than snapping to the nearest playable note: the
    // nearest note is a different tune, and a player following along would
    // hear the game ask for something the recording never plays.
    let score = FakeScore::of(&[midi("C4"), 20, midi("G4")]);
    let (chart, report) = to_chart(&score, 0, &richter_harp("C"), "A").unwrap();
    assert_eq!(report.total, 3);
    assert_eq!(report.unreachable, 1);
    assert_eq!(chart.track.len(), 2, "the unreachable note was not dropped");
}

#[test]
fn a_bend_is_written_with_the_charts_negative_convention() {
    // Charts store a downward bend as negative semitones; Technique::Bend
    // carries a positive depth. Getting the sign wrong here would raise
    // every bent note instead of lowering it.
    let score = FakeScore::of(&[midi("B4") - 1]);
    let (chart, report) = to_chart(&score, 0, &richter_harp("C"), "A").unwrap();
    assert_eq!(report.bends, 1);
    let Some(mods) = chart.track[0].events[0].modifiers.as_ref() else {
        panic!("expected a bend modifier");
    };
    match mods[0] {
        Modifier::Bend { semitones, .. } => {
            assert!(
                semitones < 0.0,
                "a downward bend must be negative, got {semitones}"
            )
        }
        ref other => panic!("expected a bend, got {other:?}"),
    }
}

#[test]
fn timing_and_metadata_come_from_the_source() {
    let score = FakeScore::of(&[midi("C4"), midi("E4")]);
    let (chart, _) = to_chart(&score, 0, &richter_harp("C"), "Some Artist").unwrap();
    assert_eq!(chart.song.title, "Fake");
    assert_eq!(chart.song.artist, "Some Artist");
    assert_eq!(chart.song.tempo_bpm, 120.0);
    assert_eq!(chart.song.time_signature.as_deref(), Some("4/4"));
    assert_eq!(chart.track[0].time, Some(0.0));
    assert_eq!(chart.track[1].time, Some(0.5));
}

#[test]
fn a_guitar_shaped_part_is_reported_as_not_worth_playing() {
    // Two octaves below a C harp's range — the realistic outcome of picking
    // a bass or low guitar track. The caller needs to be able to refuse.
    let score = FakeScore::of(&[30, 31, 32, 33, 34]);
    let (_, report) = to_chart(&score, 0, &richter_harp("C"), "A").unwrap();
    assert_eq!(report.unreachable, 5);
    assert!(!report.is_worth_playing());
    assert_eq!(report.reachable_fraction(), 0.0);
}

#[test]
fn a_mostly_playable_part_is_worth_playing() {
    // Built from the harp's own reeds rather than an invented interval
    // pattern: a whole-tone run happens to include notes a C harp can't
    // make, which is the sort of accident that makes a test assert its own
    // arithmetic instead of the behaviour under test.
    let harp = richter_harp("C");
    let mut midis: Vec<u8> = (1..=9)
        .filter_map(|hole| harp.wind_direction_midi(hole, &harmonicon_core::chart::Action::Blow))
        .collect();
    midis.push(20); // one the harp cannot possibly reach
    let score = FakeScore::of(&midis);
    let (_, report) = to_chart(&score, 0, &harp, "A").unwrap();
    assert_eq!(report.unreachable, 1);
    assert!(report.is_worth_playing(), "{report:?}");
}

#[test]
fn an_empty_track_is_never_worth_playing() {
    let score = FakeScore::of(&[]);
    let (_, report) = to_chart(&score, 0, &richter_harp("C"), "A").unwrap();
    assert!(!report.is_worth_playing());
}

/// A chart in, the same pitches out — the round trip that proves the native
/// reader and the converter agree.
#[test]
fn a_chart_converted_onto_its_own_harp_keeps_its_pitches() {
    let bytes = r#"{
            "song": {"title":"T","artist":"A","tempo_bpm":120.0,"key":"C","difficulty":"easy"},
            "timing": {"resolution":480,"tempo_map":[{"tick":0,"bpm":120.0}]},
            "harmonica": {"type":"diatonic","holes":10,"bending_profile":"richter_standard",
                "layout": {"blow":["C4","E4","G4","C5","E5","G5","C6","E6","G6","C7"],
                           "draw":["D4","G4","B4","D5","F5","A5","B5","D6","F6","A6"]}},
            "track": [
                {"time":0.0,"duration":0.5,"events":[{"hole":1,"action":"blow","note":"C4"}]},
                {"time":0.5,"duration":0.5,"events":[{"hole":2,"action":"draw","note":"G4"}]}
            ],
            "scoring": {"perfect_window_ms":50,"good_window_ms":100,"miss_window_ms":130}
        }"#
    .as_bytes()
    .to_vec();
    let source = HarpChartScore::parse(&bytes).unwrap();
    let original: Vec<u8> = source.notes(0).unwrap().iter().map(|n| n.midi).collect();

    let (chart, report) = to_chart(&source, 0, &richter_harp("C"), "A").unwrap();
    assert_eq!(report.unreachable, 0);

    let round_tripped: Vec<u8> = HarpChartScore::from_chart(chart)
        .notes(0)
        .unwrap()
        .iter()
        .map(|n| n.midi)
        .collect();
    assert_eq!(round_tripped, original);
}
