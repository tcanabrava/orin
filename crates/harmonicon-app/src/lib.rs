// SPDX-License-Identifier: MIT

//! App-wide vocabulary every feature shares: the state machine and routing
//! flags ([`app`]) and the player's persisted records ([`profile`]).
//!
//! Deliberately tiny and free of any feature: this is what gameplay, the
//! editor, jam and the menu all agree on, so anything that only one of them
//! needs belongs there instead.

pub mod app;
pub mod profile;
