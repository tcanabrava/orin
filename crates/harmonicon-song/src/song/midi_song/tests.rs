// SPDX-License-Identifier: MIT

use super::*;
use harmonicon_core::midi::note_to_midi;
use harmonicon_core::midi_file::{meta, note_off, note_on, smf_bytes};
use midly::MetaMessage;

fn midi(note: &str) -> u8 {
    note_to_midi(note).unwrap() as u8
}

/// Tracks as `(name, keys)`, each key a quarter-note.
fn midi_file(tracks: &[(&str, &[u8])]) -> Vec<u8> {
    let built: Vec<Vec<midly::TrackEvent<'static>>> = tracks
        .iter()
        .map(|(name, keys)| {
            let mut events = vec![meta(
                0,
                MetaMessage::TrackName(name.as_bytes().to_vec().leak()),
            )];
            for &key in *keys {
                events.push(note_on(0, key, 100));
                events.push(note_off(480, key));
            }
            events.push(meta(0, MetaMessage::EndOfTrack));
            events
        })
        .collect();
    smf_bytes(built)
}

/// A C-major run every C harp plays on plain blow reeds.
fn easy_notes() -> Vec<u8> {
    ["C4", "E4", "G4", "C5", "E5", "G5"].map(midi).to_vec()
}

#[test]
fn a_simple_midi_becomes_a_playable_chart() {
    let bytes = midi_file(&[("Harmonica", &easy_notes())]);
    let chart = chart_from_midi(&bytes, "Some Artist", None).unwrap();
    assert_eq!(chart.track.len(), easy_notes().len());
    assert_eq!(chart.song.artist, "Some Artist");
    assert!(
        chart.track.iter().all(|i| i.events[0].note.is_some()),
        "every converted note must state its sounding pitch"
    );
}

#[test]
fn the_track_named_harmonica_is_the_one_played() {
    // A guitar part converted to harmonica is mostly unreachable notes, so
    // picking by name is what makes a multi-track file usable at all.
    let low: Vec<u8> = (40u8..46).collect();
    let bytes = midi_file(&[("Guitar", &low), ("Harmonica", &easy_notes())]);
    let chart = chart_from_midi(&bytes, "A", None).unwrap();
    assert_eq!(chart.track.len(), easy_notes().len());
}

#[test]
fn a_single_unnamed_track_is_played_without_asking() {
    let bytes = midi_file(&[("", &easy_notes())]);
    assert!(chart_from_midi(&bytes, "A", None).is_ok());
}

#[test]
fn several_unnamed_tracks_are_refused_rather_than_guessed() {
    // Guessing "the busiest" would routinely choose a guitar. An asset
    // loader has nowhere to ask, so refusing surfaces as a load error the
    // player can act on by naming the track.
    let bytes = midi_file(&[("", &easy_notes()), ("", &easy_notes())]);
    assert!(chart_from_midi(&bytes, "A", None).is_err());
}

#[test]
fn a_part_no_harmonica_can_play_is_refused_with_a_reason() {
    // Two octaves below any harp — a bass line. The error names the counts
    // rather than saying "invalid", because the file isn't invalid.
    let bass: Vec<u8> = (28u8..40).collect();
    let bytes = midi_file(&[("Harmonica", &bass)]);
    let err = chart_from_midi(&bytes, "A", None).unwrap_err().to_string();
    assert!(
        err.contains("can be played on a harmonica"),
        "unhelpful error: {err}"
    );
}

#[test]
fn the_harmonica_is_chosen_to_fit_the_music() {
    // A tune in C should land on a C diatonic, not on whatever the default
    // happens to be.
    let harp = suggested_harp(&easy_notes());
    assert_eq!(
        harmonicon_core::harmonica::detected_harp_key(&harp).as_deref(),
        Some("C")
    );
    assert!(matches!(harp, Harmonica::Diatonic { .. }));
}

#[test]
fn a_diatonic_is_preferred_when_it_fits() {
    // A chromatic plays everything, so scoring alone would always pick one.
    // Handing a beginner a 12-hole chromatic for a tune a C diatonic plays
    // cleanly is the wrong default.
    let harp = suggested_harp(&easy_notes());
    assert!(
        matches!(harp, Harmonica::Diatonic { .. }),
        "a plainly diatonic tune chose {harp:?}"
    );
}

#[test]
fn a_chromatic_run_prefers_a_chromatic_harp() {
    // Every semitone across an octave: a diatonic can't do it, a chromatic
    // can, so the fallback has to actually fire.
    let chromatic_run: Vec<u8> = (midi("C4")..=midi("C5")).collect();
    let harp = suggested_harp(&chromatic_run);
    assert!(
        matches!(harp, Harmonica::Chromatic { .. }),
        "a fully chromatic run chose {harp:?}"
    );
}

#[test]
fn the_artist_comes_from_the_folder_that_holds_the_song() {
    // A MIDI file carries no artist, and the layout already encodes one.
    let path = std::path::Path::new("songs/Sonny Boy/Help Me/song/tune.mid");
    assert_eq!(artist_from_path(path), "Sonny Boy");
}

#[test]
fn an_unexpected_layout_falls_back_rather_than_panicking() {
    assert_eq!(
        artist_from_path(std::path::Path::new("tune.mid")),
        "Imported"
    );
}

#[test]
fn the_song_title_comes_from_its_folder_not_the_track_name() {
    // Seen on screen before this was fixed: a file whose only track is
    // named "Harmonica" produced a song called "Harmonica", because MIDI's
    // title convention is the first track's name.
    let bytes = midi_file(&[("Harmonica", &easy_notes())]);
    let chart = chart_from_midi(&bytes, "A", Some("Scale Practice".into())).unwrap();
    assert_eq!(chart.song.title, "Scale Practice");
}

#[test]
fn without_a_folder_the_files_own_title_is_kept() {
    let bytes = midi_file(&[("Harmonica", &easy_notes())]);
    let chart = chart_from_midi(&bytes, "A", None).unwrap();
    assert_eq!(chart.song.title, "Harmonica");
}

#[test]
fn the_title_is_read_from_the_song_folder() {
    let path = std::path::Path::new("songs/Sonny Boy/Help Me/song/tune.mid");
    assert_eq!(title_from_path(path).as_deref(), Some("Help Me"));
}
