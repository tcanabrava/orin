// SPDX-License-Identifier: MIT

//! Harmonicon library crate.
//!
//! Houses every subsystem so they can be shared between the game binary
//! (`src/main.rs`) and the helper tools in `src/bin/` (e.g. `hole-editor`),
//! which are separate crates and can only reach this code through the library.

pub mod app;
pub mod assets_management;
pub mod audio_system;
pub mod config_file;
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
pub mod scoring;
pub mod settings;
pub mod song;
pub mod song_editor;
pub mod spectrogram;
pub mod synthetic_dataset;
pub mod theme;
