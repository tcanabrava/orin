// SPDX-License-Identifier: MIT

//! Everything the game needs from the machine it runs on, below any
//! gameplay concept: asset discovery ([`assets_management`]), translated
//! strings ([`localization`]), persisted preferences ([`settings`]), the
//! visual theme ([`theme`]) and the narrow-window breakpoint
//! ([`responsive`]).
//!
//! Depends only on `harmonicon-core` and `harmonicon-audio`. Nothing here
//! knows what a song, a note or a screen is.

pub mod assets_management;
pub mod localization;
pub mod paths;
pub mod responsive;
pub mod settings;
pub mod theme;
