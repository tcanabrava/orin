// SPDX-License-Identifier: MIT

//! The "Jam Session" choice: pick a real song (`ArtistList`) or synthesize
//! one (`JamGenerate` — see `pages::jam_generate`).

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_fluent::Localization;

use crate::app::{GameplayMode, JamPositionCycle, JamProgression, JamScale};
use crate::jam::backing::{Genre, JamGenre};
use crate::localization::LocalizationExt;
use crate::song::chart::Scale;
use crate::song::harmonica::Progression;
use crate::theme::LoadedTheme;

use crate::menu::routing::MenuPage;
use crate::menu::scene::{spawn_back_button, spawn_button, spawn_menu_root};

pub(crate) fn setup_jam_session_menu(
    mut commands: Commands,

    theme: Res<LoadedTheme>,
    loc: Res<Localization>,
) {
    let (root, header, _page_root) = spawn_menu_root(
        &mut commands,
        &loc.msg("jam-session"),
        None,
        &theme,
        "JamSessionMenu",
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("jam-session-pick-song"),
        |_: On<Activate>,
         mut mode: ResMut<GameplayMode>,
         mut progression: ResMut<JamProgression>,
         mut scale: ResMut<JamScale>,
         mut genre: ResMut<JamGenre>,
         mut position_cycle: ResMut<JamPositionCycle>,
         mut page: ResMut<NextState<MenuPage>>| {
            *mode = GameplayMode::JamSession;
            // A real song always plays its own actual chords regardless of
            // this resource (see `twelve_bar_blues_overlay::update_bar`),
            // but reset it anyway so a stale pick from an earlier generated
            // jam can't linger and confuse anyone reading the resource value.
            progression.0 = Progression::Standard;
            // Same reasoning for scale — though unlike progression, a real
            // song's own `Harmonica::scale()` still wins over this when it
            // sets one (see `jam::session::setup`); this is only the
            // fallback for a song that doesn't.
            scale.0 = Scale::FirstPosition;
            // Same reasoning again: a real song has no genre concept
            // attached to it at all (unlike scale, no per-song override
            // exists or would make sense) — `jam::rhythm_guide` only ever
            // spawns for a `GeneratedJamSession` regardless, so this is
            // just defense against a stale value lingering.
            genre.0 = Genre::Blues;
            // Same reasoning once more: a previous jam-based lesson's
            // position-calling can't be allowed to leak into an ordinary
            // real-song jam.
            position_cycle.0 = false;
            page.set(MenuPage::ArtistList);
        },
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("jam-generate"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::JamGenerate),
    );
    spawn_back_button(
        &mut commands,
        header,
        &loc.msg("back"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Play),
    );
}
