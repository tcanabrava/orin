// SPDX-License-Identifier: MIT

//! Harmonicon library crate.
//!
//! Houses every subsystem so they can be shared between the game binary
//! (`src/main.rs`) and the helper tools in `src/bin/` (e.g. `hole-editor`),
//! which are separate crates and can only reach this code through the library.

// Re-exported from the pure-logic crate under their historical paths, so
// `crate::scoring::…` and `crate::config_file::…` read unchanged across the
// tree (see `harmonicon_core`'s own doc comment).
pub use harmonicon_core::config_file;
pub use harmonicon_core::scoring;

pub mod app;
pub mod assets_management;
pub mod audio_system;
pub mod dialogs;
pub mod gameplay;
pub mod jam;
pub mod lessons;
pub mod localization;
pub mod menu;
pub mod music_score;
pub mod note_bench;
pub mod profile;
pub mod responsive;
pub mod settings;
pub mod song;
pub mod song_editor;
pub mod spectrogram;
pub mod synthetic_dataset;
pub mod theme;
