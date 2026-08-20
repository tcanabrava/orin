// SPDX-License-Identifier: MIT

//! Microphone capture and real-time pitch detection.
//!
//! cpal callback -> mono downmix -> overlapped chunks ([`audio_input`]) ->
//! one FFT per chunk ([`pitch_detect`]), plus the offline waveform analysis
//! songs are summarised with ([`waveform`]). Depends only on
//! `harmonicon-core` for the pitch/MIDI maths; it knows nothing about
//! songs, scoring or UI.

pub mod audio_input;
pub mod config;
pub mod pipeline;
pub mod pitch_detect;
pub mod waveform;

pub use config::AudioSettings;
