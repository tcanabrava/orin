// SPDX-License-Identifier: MIT

//! Reading a playable score out of whatever file the player has.
//!
//! Harmonicon's own `.harpchart` is one format among several: a player is far
//! more likely to own a MIDI file or a Guitar Pro tab than a chart authored
//! here. [`ScoreFile`] is the single door all of them come through.
//!
//! **The native format implements the trait too.** That is deliberate: a
//! trait with one real implementation and one special case drifts, because
//! nothing forces the special case to keep fitting. Making `.harpchart` go
//! through the same door means the shape is exercised by the format we
//! control.
//!
//! Bevy-free, like `harmonicon-core` below it. Loading bytes through the
//! `AssetServer` is `harmonicon-song`'s job, a level up; this crate turns
//! bytes into notes.
//!
//! What a format has to supply is deliberately small — notes in seconds, a
//! tempo, a time signature, and a list of tracks. Everything harmonica-
//! specific (which hole, which breath, which technique) is *derived* by
//! [`convert`], using `harmonicon_core::pitch_map`, rather than being asked
//! of a format that knows nothing about harmonicas.

pub mod convert;
pub mod harpchart;
pub mod midi;
pub mod track;

pub use track::{HARMONICA_TRACK_NAMES, pick_harmonica_track};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoreError {
    #[error("not valid {format}: {detail}")]
    Parse {
        format: &'static str,
        detail: String,
    },
    #[error("this file has no tracks with any notes in it")]
    NoPlayableTracks,
    #[error("track {0} does not exist in this file")]
    NoSuchTrack(usize),
}

/// Which file format a score came from — for messages and for deciding
/// whether a track picker is worth showing at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScoreFormat {
    HarpChart,
    Midi,
}

impl ScoreFormat {
    pub fn label(self) -> &'static str {
        match self {
            ScoreFormat::HarpChart => "Harmonicon chart",
            ScoreFormat::Midi => "MIDI",
        }
    }
}

/// One playable part within a file.
///
/// `name` is what makes automatic track selection possible at all — see
/// [`pick_harmonica_track`]. It's optional because plenty of MIDI files
/// never name their tracks.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScoreTrack {
    pub index: usize,
    pub name: Option<String>,
    /// Notes in this track. Zero means it carries only tempo or metadata —
    /// common for a MIDI file's first track — and such tracks are worth
    /// hiding from a picker rather than offering as a choice that plays
    /// silence.
    pub note_count: usize,
}

impl ScoreTrack {
    pub fn is_playable(&self) -> bool {
        self.note_count > 0
    }
}

/// One note, in absolute seconds from the start of the piece.
///
/// Seconds rather than ticks because ticks are meaningless without their
/// file's own resolution and tempo map, and every format spells those
/// differently. Resolving to time in the reader keeps that variety from
/// leaking into everything downstream.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScoreNote {
    pub start_secs: f64,
    pub duration_secs: f64,
    pub midi: u8,
}

/// A score file, whatever format it arrived in.
pub trait ScoreFile {
    fn format(&self) -> ScoreFormat;

    /// The piece's title, when the format records one.
    fn title(&self) -> Option<&str>;

    /// Every track, including unplayable ones — a picker decides what to
    /// show, and hiding them here would make "track 3" ambiguous between
    /// the file's numbering and ours.
    fn tracks(&self) -> &[ScoreTrack];

    /// One track's notes, sorted by start time.
    fn notes(&self, track: usize) -> Result<Vec<ScoreNote>, ScoreError>;

    /// The nominal tempo. A real tempo map is already baked into
    /// [`ScoreNote::start_secs`]; this is for the chart's own metadata and
    /// for the metronome.
    fn tempo_bpm(&self) -> f32;

    fn time_signature(&self) -> (u8, u8);
}
