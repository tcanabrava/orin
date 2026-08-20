// SPDX-License-Identifier: MIT

//! Pitch-detection benchmarking: replays recordings through every algorithm\n//! and scores hits/misses/phantoms ([`note_bench`]), plus the synthetic\n//! dataset generator that stands in until real recordings exist\n//! ([`synthetic_dataset`]). Developer tooling, not shipped game code.

pub mod note_bench;
pub mod synthetic_dataset;
