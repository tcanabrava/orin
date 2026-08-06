// SPDX-License-Identifier: MIT

//! The main menu: Play, Options, Help, Quit.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_fluent::Localization;

use crate::localization::LocalizationExt;
use crate::theme::LoadedTheme;

use crate::menu::routing::MenuPage;
use crate::menu::scene::{spawn_button, spawn_menu_root};

pub(crate) fn setup_main_menu(
    mut commands: Commands,
    theme: Res<LoadedTheme>,
    loc: Res<Localization>,
) {
    let (root, _header, _page_root) =
        spawn_menu_root(&mut commands, &loc.msg("app-title"), None, &theme, "Main");
    spawn_button(
        &mut commands,
        root,
        &loc.msg("menu-play"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Play),
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("menu-options"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Options),
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("menu-help"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::HelpAbout),
    );
    spawn_button(
        &mut commands,
        root,
        &loc.msg("menu-quit"),
        |_: On<Activate>, mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        },
    );
}
