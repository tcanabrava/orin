// SPDX-License-Identifier: MIT

//! Song picker for the currently-selected artist. The render mode/jam
//! choice is already made by the time the player reaches this page —
//! picking a song starts the game.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_fluent::Localization;

use harmonicon_app::app::{SelectedArtist, SelectedSong};
use harmonicon_platform::assets_management::AvailableSongs;
use harmonicon_platform::localization::LocalizationExt;
use harmonicon_platform::theme::LoadedTheme;
use harmonicon_song::song::SongManifest;

use crate::menu::routing::MenuPage;
use crate::menu::scene::{spawn_back_button, spawn_button, spawn_menu_root};

pub(crate) fn setup_song_list(
    mut commands: Commands,
    songs: Res<AvailableSongs>,
    selected_artist: Res<SelectedArtist>,
    theme: Res<LoadedTheme>,
    loc: Res<Localization>,
) {
    let subtitle = format!("by {}", selected_artist.0);
    let (root, header, _page_root) = spawn_menu_root(
        &mut commands,
        &loc.msg("select-song"),
        Some(&subtitle),
        &theme,
        "SongList",
    );

    if let Some(artist_songs) = songs.0.get(&selected_artist.0) {
        let mut sorted = artist_songs.clone();
        sorted.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        for song in &sorted {
            let path = song.asset_path.clone();
            // The mode is already chosen; picking a song asks which
            // harmonica the player actually has before loading it.
            spawn_button(
                &mut commands,
                root,
                &song.name,
                move |_: On<Activate>,
                      asset_server: Res<AssetServer>,
                      mut page: ResMut<NextState<MenuPage>>,
                      mut commands: Commands| {
                    commands.insert_resource(SelectedSong(
                        asset_server.load::<SongManifest>(path.clone()),
                    ));
                    // Holding the handle starts the load, so the harp page
                    // usually has a decoded chart to price a choice against
                    // by the time it needs one.
                    page.set(MenuPage::HarpCheck);
                },
            );
        }
    }
    spawn_back_button(
        &mut commands,
        header,
        &loc.msg("back"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::ArtistList),
    );
}
