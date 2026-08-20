// SPDX-License-Identifier: MIT

//! Render-mode picker (2D/3D) shown before picking a song to play.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_fluent::Localization;

use crate::app::GameplayMode;
use harmonicon_platform::localization::LocalizationExt;
use harmonicon_platform::theme::LoadedTheme;

use crate::menu::routing::MenuPage;
use crate::menu::scene::{spawn_back_button, spawn_button, spawn_menu_root};

pub(crate) fn setup_mode_select(
    mut commands: Commands,
    theme: Res<LoadedTheme>,
    loc: Res<Localization>,
) {
    let (root, header, _page_root) = spawn_menu_root(
        &mut commands,
        &loc.msg("select-mode"),
        None,
        &theme,
        "ModeSelect",
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("play-2d"),
        |_: On<Activate>, mut mode: ResMut<GameplayMode>, mut page: ResMut<NextState<MenuPage>>| {
            *mode = GameplayMode::Play2D;
            page.set(MenuPage::ArtistList);
        },
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("play-3d"),
        |_: On<Activate>, mut mode: ResMut<GameplayMode>, mut page: ResMut<NextState<MenuPage>>| {
            *mode = GameplayMode::Play3D;
            page.set(MenuPage::ArtistList);
        },
    );
    spawn_back_button(
        &mut commands,
        header,
        &loc.msg("back"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Play),
    );
}
