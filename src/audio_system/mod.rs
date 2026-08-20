// SPDX-License-Identifier: MIT

pub mod audio_input;
pub mod config;
pub mod pipeline;
pub mod pitch_detect;
pub mod waveform;

pub use config::AudioSettings;
// Pure pitch/MIDI conversion — lives in `harmonicon-core`, re-exported so
// `crate::audio_system::midi::…` keeps resolving.
pub use harmonicon_core::{midi, synth, wav};
