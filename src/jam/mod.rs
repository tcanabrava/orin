// SPDX-License-Identifier: MIT

//! The Jam Session feature: the free-play screen and live hole-map feedback
//! ([`session`]), its improv-lesson scale-adherence accumulator
//! ([`improv`]), on-demand judging for jam-based lessons ([`lesson`]), its
//! freeform (unscored) call-and-response practice mode ([`call_response`]),
//! the live circle-of-fifths position compass ([`position_guide`]), and the
//! procedurally-generated 12-bar backing track ([`backing`]) for jamming
//! without picking an existing song.
//!
//! [`JamPlugin`] registers all of it. It used to be registered from
//! `gameplay::plugin`, which made `gameplay` depend on `jam` while `jam`
//! already depended on `gameplay` — a cycle. Composition belongs at the
//! top, so `main.rs` adds this plugin alongside `GameplayPlugin` instead
//! (`docs/physical_design_plan.md` rule 2).

use bevy::prelude::*;

use harmonicon_app::app::{AppState, GameplayMode, GeneratedJamSession};
use harmonicon_gameplay::gameplay::Paused;
use harmonicon_gameplay::gameplay::plugin::GameplayLogic;

pub mod backing;
pub mod call_response;
pub mod improv;
pub mod lesson;
pub mod midi_tracks;
pub mod position_guide;
pub mod rhythm_guide;
pub mod session;

use call_response as jam_call_response;
use midi_tracks as jam_midi_tracks;
use position_guide as jam_position_guide;
use rhythm_guide as jam_rhythm_guide;
use session as jam_session;

pub struct JamPlugin;

impl Plugin for JamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<jam_session::JamLoop>()
            .init_resource::<jam_midi_tracks::JamMidiMute>()
            .init_resource::<improv::ImprovGate>()
            .init_resource::<improv::ImprovStats>()
            .add_message::<jam_position_guide::PositionCalled>()
            .init_resource::<jam_call_response::CallResponseEnabled>()
            .init_resource::<jam_call_response::CallResponseState>()
            // Jam's own per-session reset, alongside gameplay's `reset_score`.
            .add_systems(
                OnEnter(AppState::Playing),
                (
                    improv::reset_improv_stats,
                    jam_session::setup
                        .run_if(|m: Res<GameplayMode>| *m == GameplayMode::JamSession),
                ),
            )
            // Judges a jam-based lesson when the pause menu asks.
            .add_systems(Update, lesson::finish_jam_lesson)
            // Background-music looping + the loop toggle's label.
            .add_systems(
                Update,
                (
                    jam_session::restart_finished_jam_music,
                    jam_session::update_jam_loop_label,
                )
                    .after(GameplayLogic)
                    .run_if(
                        in_state(AppState::Playing)
                            .and_then(|p: Res<Paused>| !p.0)
                            .and_then(|m: Res<GameplayMode>| *m == GameplayMode::JamSession),
                    ),
            )
            // Jam Session, position-cycling lesson mechanic: calls a new position
            // (cycling `JamScale`) every few bars and patches `JamHoleGuide` to
            // match — ordered before `improv::accumulate_improv_stats` so a
            // bar-boundary frame is never scored against the stale scale. A
            // no-op for an ordinary jam (`JamPositionCycle` off).
            .add_systems(
                Update,
                (
                    jam_position_guide::cycle_position,
                    jam_position_guide::on_position_called,
                )
                    .chain()
                    .after(GameplayLogic)
                    .before(improv::accumulate_improv_stats)
                    .run_if(
                        in_state(AppState::Playing)
                            .and_then(|p: Res<Paused>| !p.0)
                            .and_then(|m: Res<GameplayMode>| *m == GameplayMode::JamSession),
                    ),
            )
            // Jam Session: live harmonica hole-map feedback from the mic, plus the
            // improv lesson's scale-adherence tally (always accumulating during a
            // jam, not just when a lesson is in flight — same "always-on
            // diagnostic" convention as `SongStats::clean_attack`).
            .add_systems(
                Update,
                (
                    jam_session::update_hole_map,
                    improv::accumulate_improv_stats,
                    jam_call_response::drive_call_response,
                    jam_call_response::update_call_response_banner,
                    jam_call_response::update_call_response_label,
                    jam_midi_tracks::update_track_mute_buttons,
                )
                    .after(GameplayLogic)
                    .run_if(
                        in_state(AppState::Playing)
                            .and_then(|p: Res<Paused>| !p.0)
                            .and_then(|m: Res<GameplayMode>| *m == GameplayMode::JamSession),
                    ),
            )
            // Muted-track sink volume — after `apply_music_volume` (a mid-song
            // global-volume change touches every `MusicPlayer` sink, per-track
            // ones included) so a muted track always ends up silent regardless
            // of which order the two would otherwise run in.
            .add_systems(
                Update,
                jam_midi_tracks::apply_midi_track_mute
                    .after(harmonicon_gameplay::gameplay::plugin::MusicVolumeSet)
                    .run_if(
                        in_state(AppState::Playing)
                            .and_then(|m: Res<GameplayMode>| *m == GameplayMode::JamSession),
                    ),
            )
            // Jam Session: the harmonica rhythm-guide pulse row — only ever
            // spawned for a generated jam (see `jam::rhythm_guide`'s own doc
            // comment), so gated on `GeneratedJamSession`'s presence too, not
            // just the mode.
            .add_systems(
                Update,
                jam_rhythm_guide::update_rhythm_guide
                    .after(GameplayLogic)
                    .run_if(
                        in_state(AppState::Playing)
                            .and_then(|p: Res<Paused>| !p.0)
                            .and_then(|m: Res<GameplayMode>| *m == GameplayMode::JamSession)
                            .and_then(resource_exists::<GeneratedJamSession>),
                    ),
            );
    }
}
