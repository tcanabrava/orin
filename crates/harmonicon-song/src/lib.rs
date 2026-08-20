// SPDX-License-Identifier: MIT

//! Playable content: the chart/manifest asset pipeline ([`song`]) and the
//! curriculum built on top of it ([`lessons`]).
//!
//! `harmonicon-core` owns the chart *types*; this crate owns loading them
//! through Bevy's `AssetServer`, decoding a song's sibling audio, and
//! discovering lessons on disk. Above core/audio/platform, below anything
//! that draws or scores.

pub mod lessons;
pub mod song;
