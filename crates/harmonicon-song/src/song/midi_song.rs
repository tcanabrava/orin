// SPDX-License-Identifier: MIT

//! Playing a `.mid` file directly, without authoring a chart first.
//!
//! A player is far likelier to own a MIDI file than a `.harpchart`. This
//! makes one droppable into `~/Harmonicon/songs/<artist>/<song>/song/` and
//! playable, by turning it into a chart at asset-load time through
//! `harmonicon_score`.
//!
//! **`song/music.mid` already meant something else** — the backing audio for
//! a `.harpchart` song, rendered per-track by `loader::load_midi_tracks`.
//! Both readings of the same extension are legitimate, so discovery
//! (`assets_management::scan_all_songs`) prefers a `.harpchart` and only
//! treats a MIDI as the chart when there is no chart beside it.
//!
//! Which harmonica the chart is built for is *chosen*, not assumed: a MIDI
//! file says nothing about harmonicas, so `pitch_map::suggest_key` picks the
//! key needing the fewest bends — exactly what the Song Editor's own import
//! does. The player can change it on the harp-check screen before playing,
//! and that screen's cost readout is what tells them whether the guess was
//! any good.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use harmonicon_core::chart::HarpChart;
use harmonicon_core::harmonica::Harmonica;
use harmonicon_core::pitch_map::{HarpKind, harp_for_key, suggest_key};
use harmonicon_score::midi::MidiScore;
use harmonicon_score::{ScoreFile, convert, pick_harmonica_track};

use super::SongManifest;
use super::loader::{SongLoadError, assemble_manifest};

/// Fraction of a track's notes that must land on the harp before the result
/// is offered as a song.
///
/// Below this the chart is mostly holes — the realistic outcome of a file
/// whose only parts are guitar and bass. Refusing is better than a song that
/// looks playable and isn't; `ConversionReport` carries the same judgement
/// for callers that want to explain it.
const MIN_REACHABLE: f32 = 0.8;

#[derive(Default, TypePath)]
pub struct MidiSongLoader;

impl AssetLoader for MidiSongLoader {
    type Asset = SongManifest;
    type Settings = ();
    type Error = SongLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<SongManifest, SongLoadError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let path = load_context.path().path();
        let chart = chart_from_midi(&bytes, &artist_from_path(path), title_from_path(path))?;
        assemble_manifest(chart, load_context).await
    }

    fn extensions(&self) -> &[&str] {
        &["mid", "midi"]
    }
}

/// The artist folder a song sits in — `songs/<artist>/<song>/song/x.mid`.
///
/// The file itself carries no artist, and the folder layout already encodes
/// one, so reading it back is better than writing "Unknown" into every
/// imported chart.
fn artist_from_path(path: &std::path::Path) -> String {
    path.ancestors()
        .nth(3)
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported".to_string())
}

/// The song folder's name — `songs/<artist>/<song>/song/x.mid`.
///
/// Preferred over the file's own title because MIDI's convention is that the
/// title is the *first track's* name, and a harmonica file's first track is
/// usually named "Harmonica". Left as `None` when the layout doesn't match,
/// so the file's own title is still used rather than a guess.
fn title_from_path(path: &std::path::Path) -> Option<String> {
    path.ancestors()
        .nth(2)
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
}

/// Turns MIDI bytes into a chart, choosing the track and the harmonica.
pub fn chart_from_midi(
    bytes: &[u8],
    artist: &str,
    title: Option<String>,
) -> Result<HarpChart, SongLoadError> {
    let score =
        MidiScore::parse(bytes.to_vec()).map_err(|e| SongLoadError::Validation(e.to_string()))?;

    let track = choose_track(&score).ok_or_else(|| {
        SongLoadError::Validation("this MIDI file has no track with any notes in it".to_string())
    })?;

    let notes = score
        .notes(track)
        .map_err(|e| SongLoadError::Validation(e.to_string()))?;
    let harp = suggested_harp(&notes.iter().map(|n| n.midi).collect::<Vec<_>>());

    let (mut chart, report) = convert::to_chart(&score, track, &harp, artist)
        .map_err(|e| SongLoadError::Validation(e.to_string()))?;
    if let Some(title) = title {
        chart.song.title = title;
    }

    if report.reachable_fraction() < MIN_REACHABLE {
        return Err(SongLoadError::Validation(format!(
            "only {reached} of {total} notes in this track can be played on a harmonica — \
             it is probably not a harmonica part. Name the track \"Harmonica\" if it is one.",
            reached = report.total - report.unreachable,
            total = report.total,
        )));
    }
    Ok(chart)
}

/// The track to play: one named for a harmonica, else the only playable
/// track there is.
///
/// Deliberately *not* "the busiest track" when nothing is named: that would
/// routinely pick a guitar part and produce a chart no one can play. With
/// several unnamed candidates this refuses, which surfaces as a load error
/// rather than a silently wrong choice — a picker belongs in the UI, and
/// an asset loader has nowhere to ask.
fn choose_track(score: &MidiScore) -> Option<usize> {
    if let Some(named) = pick_harmonica_track(score.tracks()) {
        return Some(named);
    }
    let mut playable = score.tracks().iter().filter(|t| t.is_playable());
    let only = playable.next()?;
    playable.next().is_none().then_some(only.index)
}

/// The harmonica a set of pitches fits best.
///
/// Tries diatonic first and only prefers a chromatic when it genuinely fits
/// better: a chromatic can play everything, so scoring alone would always
/// choose one, and handing a beginner a 12-hole chromatic for a tune a C
/// diatonic plays cleanly is the wrong default.
pub fn suggested_harp(pitches: &[u8]) -> Harmonica {
    let diatonic_key = suggest_key(pitches, HarpKind::Diatonic);
    let diatonic = harp_for_key(diatonic_key, HarpKind::Diatonic);
    if reachable(pitches, &diatonic) >= MIN_REACHABLE {
        return diatonic;
    }
    let chromatic_key = suggest_key(pitches, HarpKind::Chromatic);
    let chromatic = harp_for_key(chromatic_key, HarpKind::Chromatic);
    if reachable(pitches, &chromatic) > reachable(pitches, &diatonic) {
        chromatic
    } else {
        diatonic
    }
}

fn reachable(pitches: &[u8], harp: &Harmonica) -> f32 {
    if pitches.is_empty() {
        return 0.0;
    }
    let hit = pitches
        .iter()
        .filter(|&&p| harmonicon_core::pitch_map::map_pitch_playable(p, harp).is_some())
        .count();
    hit as f32 / pitches.len() as f32
}

#[cfg(test)]
mod tests;
