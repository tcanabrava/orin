// SPDX-License-Identifier: MIT

//! Generated Jam Session setup: pick a key and tempo, then start an
//! endless synthesized 12-bar backing (`crate::jam::backing`) without first
//! picking an existing song — a second way into `GameplayMode::JamSession`
//! alongside the "Jam Session" button's real-song flow.

use bevy::audio::AudioSource;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_fluent::Localization;

use crate::audio_system::midi::NOTE_NAMES;
use crate::dialogs::combobox;
use crate::dialogs::text_input::{NumericInputCommitted, spawn_numeric_input};
use crate::jam::backing::{GeneratedJamSession, build_generated_manifest};
use crate::localization::LocalizationExt;
use crate::song::SongManifest;
use crate::song::harmonica::{Position, Progression};
use crate::theme::LoadedTheme;

use crate::app::{AppState, GameplayMode, JamProgression, SelectedSong};
use crate::menu::routing::MenuPage;
use crate::menu::scene::{spawn_back_button, spawn_button, spawn_menu_root};

const MIN_BPM: f32 = 60.0;
const MAX_BPM: f32 = 160.0;

/// The key/tempo currently selected on this page. Persists across visits
/// (like `bending_trainer::TrainerKey`/`TrainerTarget`), so re-opening the
/// page keeps your last choice instead of resetting to the default.
#[derive(Resource)]
pub(crate) struct JamGenerateConfig {
    pub key: String,
    pub bpm: f32,
    pub progression: Progression,
    pub position: Position,
}

impl Default for JamGenerateConfig {
    fn default() -> Self {
        Self {
            key: "C".to_string(),
            bpm: 90.0,
            progression: Progression::Standard,
            position: Position::First,
        }
    }
}

fn key_labels() -> Vec<String> {
    NOTE_NAMES.iter().map(|s| s.to_string()).collect()
}

fn progression_labels() -> Vec<String> {
    Progression::all()
        .iter()
        .map(|p| p.label().to_string())
        .collect()
}

fn position_labels() -> Vec<String> {
    Position::all()
        .iter()
        .map(|p| p.label().to_string())
        .collect()
}

pub(crate) fn setup_jam_generate_menu(
    mut commands: Commands,
    config: Res<JamGenerateConfig>,
    theme: Res<LoadedTheme>,
    loc: Res<Localization>,
) {
    let (root, header, page_root) = spawn_menu_root(
        &mut commands,
        &loc.msg("jam-generate-title"),
        None,
        &theme,
        "JamGenerate",
    );

    combobox::spawn_combobox(
        &mut commands,
        root,
        page_root,
        &loc.msg("jam-generate-key"),
        &key_labels(),
        &config.key,
        |ev: On<combobox::ComboboxSelect>, mut cfg: ResMut<JamGenerateConfig>| {
            cfg.key = ev.value.clone();
        },
    );

    combobox::spawn_combobox(
        &mut commands,
        root,
        page_root,
        &loc.msg("jam-generate-progression"),
        &progression_labels(),
        config.progression.label(),
        |ev: On<combobox::ComboboxSelect>, mut cfg: ResMut<JamGenerateConfig>| {
            if let Some(p) = Progression::from_label(&ev.value) {
                cfg.progression = p;
            }
        },
    );

    combobox::spawn_combobox(
        &mut commands,
        root,
        page_root,
        &loc.msg("jam-generate-position"),
        &position_labels(),
        config.position.label(),
        |ev: On<combobox::ComboboxSelect>, mut cfg: ResMut<JamGenerateConfig>| {
            if let Some(p) = Position::from_label(&ev.value) {
                cfg.position = p;
            }
        },
    );

    // ── Tempo: a numeric text box rather than a combobox — a free-form BPM
    // isn't a short pick-one-of-N choice, it's a number, so it gets its own
    // widget (`dialogs::text_input::spawn_numeric_input`) instead of forcing
    // it into the same "pick from a list" shape as Key/Progression/Position.
    let tempo_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .id();
    commands.entity(root).add_child(tempo_row);
    commands.entity(tempo_row).with_children(|row| {
        row.spawn((
            Text::new(String::from(loc.msg("jam-generate-tempo"))),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    spawn_numeric_input(
        &mut commands,
        tempo_row,
        config.bpm,
        MIN_BPM,
        MAX_BPM,
        Color::srgb(0.10, 0.10, 0.16),
        Color::srgb(0.35, 0.35, 0.45),
        |ev: On<NumericInputCommitted>, mut cfg: ResMut<JamGenerateConfig>| {
            cfg.bpm = ev.value;
        },
    );

    spawn_button(
        &mut commands,
        root,
        &loc.msg("jam-generate-start"),
        |_: On<Activate>,
         config: Res<JamGenerateConfig>,
         theme: Res<LoadedTheme>,
         mut manifests: ResMut<Assets<SongManifest>>,
         mut sources: ResMut<Assets<AudioSource>>,
         mut mode: ResMut<GameplayMode>,
         mut progression: ResMut<JamProgression>,
         mut commands: Commands,
         mut state: ResMut<NextState<AppState>>| {
            let background = theme.default_background.clone().unwrap_or_default();
            let manifest = build_generated_manifest(
                &config.key,
                config.bpm,
                config.progression,
                config.position,
                background,
                Handle::default(),
                &mut sources,
            );
            let handle = manifests.add(manifest);
            commands.insert_resource(SelectedSong(handle));
            commands.insert_resource(GeneratedJamSession);
            *mode = GameplayMode::JamSession;
            progression.0 = config.progression;
            // Synthesized synchronously above (no async asset load to wait
            // on), so this skips `AppState::SongLoading` entirely and goes
            // straight to `Playing` — `check_loading`'s only job is waiting
            // on `asset_server.is_loaded_with_dependencies`, which a
            // manifest built by `Assets::add` (not `AssetServer::load`)
            // never needs (and, per `GeneratedJamSession`'s doc comment,
            // never gets — `on_restart` skips `SongLoading` the same way).
            state.set(AppState::Playing);
        },
    );

    spawn_back_button(
        &mut commands,
        header,
        &loc.msg("back"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::JamSessionMenu),
    );
}
