// SPDX-License-Identifier: MIT

//! Reusable presentation with no gameplay knowledge: the widget library
//! ([`dialogs`] — buttons, comboboxes, file/confirm dialogs, page chrome),
//! the SMuFL notation staff ([`music_score`]) and the live audio
//! visualiser ([`spectrogram`]).
//!
//! A widget here must work for any caller. Anything that knows what a note,
//! a score or a lesson is belongs in the feature that owns it.

pub mod dialogs;
pub mod music_score;
pub mod spectrogram;
