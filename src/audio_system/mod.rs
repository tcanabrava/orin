// SPDX-License-Identifier: MIT

pub mod audio_input;
pub mod config;
pub mod midi;
pub mod pipeline;
pub mod pitch_detect;
pub(crate) mod synth;
pub mod wav;
pub mod waveform;

pub use config::AudioSettings;
