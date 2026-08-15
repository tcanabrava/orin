// SPDX-License-Identifier: MIT

//! The Jam Session feature: the free-play screen and live hole-map feedback
//! ([`session`]), its improv-lesson scale-adherence accumulator
//! ([`improv`]), its freeform (unscored) call-and-response practice mode
//! ([`call_response`]), the live circle-of-fifths position compass
//! ([`position_guide`]), and the procedurally-generated 12-bar backing track
//! ([`backing`]) for jamming without picking an existing song.

pub mod backing;
pub mod call_response;
pub mod improv;
pub mod midi_tracks;
pub mod position_guide;
pub mod rhythm_guide;
pub mod session;
