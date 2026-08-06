// SPDX-License-Identifier: MIT

//! The Options page: audio volume sliders plus 2D-note / 3D-note / harmonica
//! pickers with live previews. The 3D previews render each model to an off-screen
//! texture (one render layer per preview) shown as a UI image. Owns its page
//! lifecycle via [`OptionsPlugin`]; the menu shell only routes to it.

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::IntoObserverSystem;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::Pickable;
use bevy::picking::events::{Out, Over, Pointer};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::ui_widgets::Button as WidgetButton;
use bevy::ui_widgets::{
    Activate, Slider, SliderRange, SliderStep, SliderValue, TrackClick, ValueChange,
    slider_self_update,
};
use bevy_fluent::Localization;

const TRACK_BG: Color = Color::srgb(0.14, 0.14, 0.22);

use crate::assets_management::{AvailableHarmonicas, SelectedHarmonicaModel, ShowNoteNumbers};
use crate::audio_system::audio_input::{self, MicStatus};
use crate::localization::LocalizationExt;
use crate::settings::AudioSettings;

use crate::theme::LoadedTheme;

use crate::app::AppState;
use crate::menu::routing::MenuPage;
use crate::menu::scene::{MenuRoot, cleanup_menu, spawn_back_button, spawn_button, spawn_menu_root};

use crate::dialogs::algo_picker::{algo_labels, attach_algo_tooltip, on_algo_selected};
use crate::dialogs::button;
use crate::dialogs::checkbox;
use crate::dialogs::combobox;
use crate::dialogs::tooltip::Tooltip;

/// Owns the Options page: builds it on entry, tears it down on exit, and runs
/// the slider/preview interaction systems while it's open.
pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuPage::Options), setup_options_menu)
            .add_systems(OnExit(MenuPage::Options), cleanup_menu)
            // Keep each slider's own SliderValue in sync as it's dragged or
            // stepped, so keyboard adjustment works from the current value.
            .add_observer(slider_self_update)
            // Sliders and harmonica buttons carry their own change/click/hover
            // behaviour as inline on(...) observers; these systems only mirror
            // settings/selection onto the visuals.
            .add_systems(
                Update,
                (
                    update_sliders,
                    update_latency_slider,
                    harmonica_button_visuals,
                    propagate_preview_layers,
                    update_mic_banner,
                    sync_mic_combobox,
                    update_zoom_slider_visuals,
                    sync_zoom_slider_from_ui_scale,
                )
                    .run_if(in_state(MenuPage::Options)),
            );
    }
}

// ── Components ──────────────────────────────────────────────────────────────

/// Which audio level a slider controls.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
enum VolumeSlider {
    #[default]
    Music,
    Metronome,
}

/// The growing fill of a slider track; its width mirrors the bound level.
#[derive(Component, Default, Clone)]
struct SliderFill(VolumeSlider);

/// The "NN%" readout beside a slider.
#[derive(Component)]
struct SliderValueLabel(VolumeSlider);

/// A harmonica-model choice button; carries the model name.
#[derive(Component, Default, Clone)]
struct HarmonicaButton(String);

/// The "no microphone" warning banner, hidden only while
/// [`MicStatus::Connected`] — see [`mic_banner_visible`]. See TODO.md: "No
/// microphone = silent failure."
#[derive(Component)]
struct MicBanner;

/// The failure-reason text inside [`MicBanner`].
#[derive(Component)]
struct MicBannerText;

/// Marks a preview scene root (a `WorldAssetRoot`); the propagation system forces
/// this `RenderLayers` onto all its descendants, since glTF scene children don't
/// inherit it and would otherwise be invisible to the preview camera.
#[derive(Component)]
struct PreviewSceneLayer(RenderLayers);

/// Marks the drag track of the input-latency slider.
#[derive(Component, Default, Clone)]
struct LatencySlider;

/// The fill bar inside the latency slider track.
#[derive(Component, Default, Clone)]
struct LatencySliderFill;

/// The "Xms" readout beside the latency slider.
#[derive(Component)]
struct LatencySliderLabel;

/// The "Zoom: N%" readout beside the zoom slider.
#[derive(Component)]
struct ZoomLabel;

/// Current level for a given slider kind.
fn audio_level(settings: &AudioSettings, kind: VolumeSlider) -> f32 {
    match kind {
        VolumeSlider::Music => settings.music_volume,
        VolumeSlider::Metronome => settings.metronome_volume,
    }
}

// ── Page setup ────────────────────────────────────────────────────────────────

fn setup_options_menu(
    mut commands: Commands,
    loc: Res<Localization>,
    settings: Res<AudioSettings>,
    mic_status: Res<MicStatus>,
    harmonicas: Res<AvailableHarmonicas>,
    selected_harmonica: Res<SelectedHarmonicaModel>,
    asset_server: Res<AssetServer>,
    images: ResMut<Assets<Image>>,
    theme: Res<LoadedTheme>,
    show_numbers: Res<ShowNoteNumbers>,
    adaptive_difficulty: Res<crate::settings::AdaptiveDifficultyEnabled>,
    fullscreen: Res<crate::settings::FullscreenEnabled>,
    colorblind_palette: Res<crate::settings::ColorblindPalette>,
    action_button_style: Res<crate::settings::ActionButtonStyle>,
    ui_scale: Res<UiScale>,
) {
    let (root, header, page_root) =
        spawn_menu_root(&mut commands, "Options", Some("Audio"), &theme, "Options");

    spawn_back_button(
        &mut commands,
        header,
        &loc.msg("options-back-tooltip"),
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Main),
    );

    // Two columns, sized to their own content (like every other menu page's
    // body) rather than a fixed screen percentage — `root` here is the
    // shared scrollable body area (`menu::scene::spawn_menu_root`), which
    // itself sizes to *its* content and gets centered as a whole, so a
    // percentage width/height on `main_layout` would resolve against that
    // auto-sized ancestor instead of the actual screen, producing an
    // arbitrary, off-center result. Left undefined, `main_layout` and its
    // two columns naturally size to their own content and are centered by
    // `root`'s own centering — same mechanism every single-column page
    // already relies on.
    let main_layout = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            ..default()
        })
        .id();

    let left_layout = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            // `row_gap`, not `column_gap` — this stacks rows vertically, so
            // the gap that matters is between rows, along the main axis.
            row_gap: Val::Px(20.0),
            ..default()
        })
        .id();

    let right_layout = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .id();

    commands.entity(root).add_child(main_layout);
    commands.entity(main_layout).add_child(left_layout);
    commands.entity(main_layout).add_child(right_layout);

    spawn_left_column(
        &mut commands,
        left_layout,
        page_root,
        mic_status,
        settings,
        harmonicas,
        &loc,
        selected_harmonica,
        asset_server,
        images,
        show_numbers,
        adaptive_difficulty,
        fullscreen,
        colorblind_palette,
        *action_button_style,
        ui_scale.0,
    );
    spawn_right_column(&mut commands, right_layout, &loc);
}

fn spawn_left_column(
    commands: &mut Commands,
    parent: Entity,
    page_root: Entity,
    mic_status: Res<MicStatus>,
    settings: Res<AudioSettings>,
    harmonicas: Res<AvailableHarmonicas>,
    loc: &Localization,
    selected_harmonica: Res<SelectedHarmonicaModel>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    show_numbers: Res<ShowNoteNumbers>,
    adaptive_difficulty: Res<crate::settings::AdaptiveDifficultyEnabled>,
    fullscreen: Res<crate::settings::FullscreenEnabled>,
    colorblind_palette: Res<crate::settings::ColorblindPalette>,
    action_button_style: crate::settings::ActionButtonStyle,
    ui_scale: f32,
) {
    spawn_mic_banner(commands, parent, &mic_status, loc);
    spawn_volume_slider(
        commands,
        parent,
        "Music",
        &loc.msg("options-music-volume-tooltip"),
        VolumeSlider::Music,
        settings.music_volume,
        set_music_volume,
    );
    spawn_volume_slider(
        commands,
        parent,
        "Metronome",
        &loc.msg("options-metronome-volume-tooltip"),
        VolumeSlider::Metronome,
        settings.metronome_volume,
        set_metronome_volume,
    );
    spawn_latency_slider(commands, parent, settings.input_latency_ms, loc);
    spawn_mic_combobox(
        commands,
        parent,
        page_root,
        loc,
        &audio_input::input_device_names(),
        connected_device_name(&mic_status),
    );

    // Harmonica previews: the model's glTF scene rendered to a texture (its own
    // materials, no tint). Layers are assigned after the 3D-note layers so the
    // preview cameras never capture each other's models.
    let harmonica_base_layer = 1;
    let previews_harmonica: Vec<(Handle<Image>, String)> = harmonicas
        .0
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let handle = spawn_harmonica_preview(
                commands,
                &mut images,
                &asset_server,
                m,
                harmonica_base_layer + i,
            );
            (handle, m.clone())
        })
        .collect();

    spawn_harmonica_row(
        commands,
        parent,
        loc,
        &previews_harmonica,
        &selected_harmonica.0,
    );

    let algo_combo = combobox::spawn_combobox(
        commands,
        parent,
        page_root,
        &loc.msg("options-pitch-detect"),
        &algo_labels(),
        settings.pitch_algorithm.label(),
        on_algo_selected,
    );
    attach_algo_tooltip(commands, algo_combo, settings.pitch_algorithm);

    spawn_note_numbers_toggle(commands, parent, loc, show_numbers.0);
    spawn_adaptive_difficulty_toggle(commands, parent, adaptive_difficulty.0, loc);
    spawn_fullscreen_toggle(commands, parent, fullscreen.0, loc);
    spawn_colorblind_palette_toggle(commands, parent, colorblind_palette.0, loc);
    spawn_zoom_slider(commands, parent, ui_scale, loc);
    spawn_action_button_style_combobox(commands, parent, page_root, loc, action_button_style);
}

fn spawn_right_column(commands: &mut Commands, parent: Entity, loc: &Localization) {
    let theme_btn = spawn_button(
        commands,
        parent,
        "Theme",
        |_: On<Activate>, mut page: ResMut<NextState<MenuPage>>| page.set(MenuPage::Theme),
    );
    commands
        .entity(theme_btn)
        .insert(Tooltip(String::from(loc.msg("options-theme-tooltip"))));

    let calibrate_btn = spawn_button(
        commands,
        parent,
        &loc.msg("options-calibrate-input-lag"),
        |_: On<Activate>, mut state: ResMut<NextState<AppState>>| state.set(AppState::Calibration),
    );
    commands.entity(calibrate_btn).insert(Tooltip(String::from(
        loc.msg("options-calibrate-input-lag-tooltip"),
    )));
}

/// Flips whether falling notes show their hole number instead of the
/// blow/draw arrow (`gameplay_2d`/`gameplay_3d`'s note spawners read this).
fn set_note_numbers(ev: On<ValueChange<bool>>, mut show: ResMut<ShowNoteNumbers>) {
    show.0 = ev.value;
}

/// A checkbox bound to [`ShowNoteNumbers`], with a tooltip explaining the
/// two display modes it switches between.
fn spawn_note_numbers_toggle(
    commands: &mut Commands,
    parent: Entity,
    loc: &Localization,
    show_numbers: bool,
) {
    let row = checkbox::spawn_checkbox(
        commands,
        parent,
        &loc.msg("options-note-labels"),
        show_numbers,
        set_note_numbers,
    );
    commands.entity(row).insert(Tooltip(String::from(
        loc.msg("options-note-labels-tooltip"),
    )));
}

/// Flips the single global adaptive-difficulty setting — not per-song, see
/// `settings::AdaptiveDifficultyEnabled`'s doc comment. Persisted
/// automatically by `settings`'s debounced-save machinery, same as every
/// other Options-page toggle; doesn't touch the live per-session
/// `gameplay::adaptive_difficulty::AdaptiveDifficulty` cache — that only
/// gets (re)seeded from this setting at the next song's start, or flipped
/// directly by the pause menu's own toggle for an immediate mid-song effect.
fn set_adaptive_difficulty(
    ev: On<ValueChange<bool>>,
    mut enabled: ResMut<crate::settings::AdaptiveDifficultyEnabled>,
) {
    enabled.0 = ev.value;
}

/// A checkbox bound to the global adaptive-difficulty setting — see
/// `settings::AdaptiveDifficultyEnabled`'s doc comment for what it does and
/// how it interacts with the pause menu's own live toggle.
fn spawn_adaptive_difficulty_toggle(
    commands: &mut Commands,
    parent: Entity,
    enabled: bool,
    loc: &Localization,
) {
    let row = checkbox::spawn_checkbox(
        commands,
        parent,
        &loc.msg("options-adaptive-difficulty"),
        enabled,
        set_adaptive_difficulty,
    );
    commands.entity(row).insert(Tooltip(String::from(
        loc.msg("options-adaptive-difficulty-tooltip"),
    )));
}

/// Flips the fullscreen preference; `settings::apply_fullscreen` mirrors the
/// resulting `FullscreenEnabled` onto the primary window's `WindowMode`.
fn set_fullscreen(
    ev: On<ValueChange<bool>>,
    mut fullscreen: ResMut<crate::settings::FullscreenEnabled>,
) {
    fullscreen.0 = ev.value;
}

/// A checkbox bound to the fullscreen setting.
fn spawn_fullscreen_toggle(
    commands: &mut Commands,
    parent: Entity,
    enabled: bool,
    loc: &Localization,
) {
    let row = checkbox::spawn_checkbox(
        commands,
        parent,
        &loc.msg("options-fullscreen"),
        enabled,
        set_fullscreen,
    );
    commands
        .entity(row)
        .insert(Tooltip(String::from(loc.msg("options-fullscreen-tooltip"))));
}

/// Marks the zoom slider's own track, so its `SliderValue` can be told apart
/// from the volume/latency sliders', which also carry one.
#[derive(Component, Default, Clone)]
struct ZoomSlider;

/// The fill bar inside the zoom slider track.
#[derive(Component, Default, Clone)]
struct ZoomSliderFill;

fn zoom_label_text(loc: &Localization, scale: f32) -> String {
    loc.msg_args(
        "options-zoom-label",
        &[("percent", (scale * 100.0).round().to_string())],
    )
    .into()
}

/// Where `scale` sits between `dialogs::ui_scale`'s `MIN_SCALE`/`MAX_SCALE`,
/// as a `0.0..=1.0` fraction — shared by the slider's initial spawn position
/// and its live fill width.
fn zoom_fraction(scale: f32) -> f32 {
    use crate::dialogs::ui_scale::{MAX_SCALE, MIN_SCALE};
    ((scale - MIN_SCALE) / (MAX_SCALE - MIN_SCALE)).clamp(0.0, 1.0)
}

/// Commits the dragged/stepped value to the real `UiScale` only once the
/// interaction is finished (`is_final`), never on every drag frame:
/// `UiScale` changing forces Bevy to re-rasterize every visible glyph at
/// the new effective size, and applying that continuously mid-drag risks
/// the same GPU-memory-exhaustion crash `dialogs::ui_scale`'s doc comment
/// describes for the keyboard shortcut. The live drag preview comes from
/// `SliderValue` instead (mirrored onto the fill/label by
/// [`update_zoom_slider_visuals`]), which costs nothing to update every
/// frame.
fn set_zoom(ev: On<ValueChange<f32>>, mut ui_scale: ResMut<UiScale>) {
    if ev.is_final {
        ui_scale.0 = ev.value;
    }
}

/// A labelled zoom slider — the only way to change `UiScale`; an earlier
/// Arrow Up/Down keyboard shortcut was removed because it conflicted with
/// Tab/arrow-key UI navigation.
fn spawn_zoom_slider(commands: &mut Commands, parent: Entity, scale: f32, loc: &Localization) {
    use crate::dialogs::ui_scale::{MAX_SCALE, MIN_SCALE};

    let label = String::from(loc.msg("options-zoom"));
    let tooltip = String::from(loc.msg("options-zoom-tooltip"));
    let row = spawn_slider_row(commands, parent, &label, &tooltip);
    let frac = zoom_fraction(scale);

    let track = commands
        .spawn_scene(zoom_slider_scene(scale, frac))
        .insert((SliderRange::new(MIN_SCALE, MAX_SCALE), SliderStep(0.1)))
        .id();
    commands.entity(row).add_child(track);

    spawn_slider_value_label(commands, row, zoom_label_text(loc, scale), ZoomLabel);
}

/// The zoom slider track: a `bsn!` `Slider` + fill, wired to [`set_zoom`].
fn zoom_slider_scene(value: f32, frac: f32) -> impl Scene {
    bsn! {
        Slider { track_click: {TrackClick::Snap} }
        TabIndex(0)
        SliderValue({value})
        Node { width: {Val::Px(220.0)}, height: {Val::Px(14.0)} }
        BackgroundColor({TRACK_BG})
        ZoomSlider
        on(set_zoom)
        Children [
            (
                Node { width: {Val::Percent(frac * 100.0)}, height: {Val::Percent(100.0)} }
                BackgroundColor({Color::srgb(0.55, 0.45, 0.85)})
                ZoomSliderFill
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            )
        ]
    }
}

/// Mirrors the zoom slider's own live `SliderValue` (updated every drag
/// frame by `slider_self_update`, regardless of `is_final`) onto its fill
/// and "Zoom: N%" label — safe to run continuously, unlike touching
/// `UiScale` itself (see [`set_zoom`]).
fn update_zoom_slider_visuals(
    loc: Res<Localization>,
    sliders: Query<&SliderValue, (With<ZoomSlider>, Changed<SliderValue>)>,
    mut fills: Query<&mut Node, With<ZoomSliderFill>>,
    mut labels: Query<&mut Text, With<ZoomLabel>>,
) {
    let Ok(value) = sliders.single() else {
        return;
    };
    for mut node in &mut fills {
        node.width = Val::Percent(zoom_fraction(value.0) * 100.0);
    }
    for mut text in &mut labels {
        *text = Text::new(zoom_label_text(&loc, value.0));
    }
}

/// A combobox picking `settings::ActionButtonStyle` — how the Song Editor's
/// action buttons render (icon only, icon + text, or text only).
fn spawn_action_button_style_combobox(
    commands: &mut Commands,
    parent: Entity,
    page_root: Entity,
    loc: &Localization,
    current: crate::settings::ActionButtonStyle,
) {
    let options: Vec<String> = crate::settings::ActionButtonStyle::all()
        .iter()
        .map(|s| String::from(loc.msg(s.loc_key())))
        .collect();
    let combo = combobox::spawn_combobox(
        commands,
        parent,
        page_root,
        &loc.msg("options-button-style"),
        &options,
        &loc.msg(current.loc_key()),
        on_action_button_style_selected,
    );
    commands.entity(combo).insert(Tooltip(String::from(
        loc.msg("options-button-style-tooltip"),
    )));
}

fn on_action_button_style_selected(
    ev: On<combobox::ComboboxSelect>,
    loc: Res<Localization>,
    mut style: ResMut<crate::settings::ActionButtonStyle>,
) {
    if let Some(picked) = crate::settings::ActionButtonStyle::from_localized_label(&loc, &ev.value)
    {
        *style = picked;
    }
}

/// Keeps the slider's own `SliderValue` in step with `UiScale` when it
/// changes from outside the slider (the Arrow Up/Down shortcut) — otherwise
/// the slider would silently drift out of sync with the actual scale until
/// next dragged. `SliderValue` is an immutable component (replace via
/// `insert`, not `&mut`), same as every other `bevy_ui_widgets` value type.
fn sync_zoom_slider_from_ui_scale(
    ui_scale: Res<UiScale>,
    sliders: Query<Entity, With<ZoomSlider>>,
    mut commands: Commands,
) {
    if !ui_scale.is_changed() {
        return;
    }
    for entity in &sliders {
        commands.entity(entity).insert(SliderValue(ui_scale.0));
    }
}

/// Flips whether scored notes use the fixed colorblind-safe blow/draw pair
/// instead of the active theme's own note colors — see
/// `settings::ColorblindPalette`'s doc comment.
fn set_colorblind_palette(
    ev: On<ValueChange<bool>>,
    mut enabled: ResMut<crate::settings::ColorblindPalette>,
) {
    enabled.0 = ev.value;
}

/// A checkbox bound to the colorblind-palette setting.
fn spawn_colorblind_palette_toggle(
    commands: &mut Commands,
    parent: Entity,
    enabled: bool,
    loc: &Localization,
) {
    let row = checkbox::spawn_checkbox(
        commands,
        parent,
        &loc.msg("options-colorblind-palette"),
        enabled,
        set_colorblind_palette,
    );
    commands.entity(row).insert(Tooltip(String::from(
        loc.msg("options-colorblind-palette-tooltip"),
    )));
}

/// A labelled row of harmonica-model choice buttons, each showing a rendered
/// preview above its name. Each button is a `bsn!` scene carrying its own
/// dedicated "select this model" click callback plus hover;
fn spawn_harmonica_row(
    commands: &mut Commands,
    parent: Entity,
    loc: &Localization,
    previews: &[(Handle<Image>, String)],
    selected: &str,
) {
    let row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .id();

    commands.entity(row).with_children(|r| {
        r.spawn((
            Node {
                width: Val::Px(110.0),
                ..default()
            },
            Text::new("Harmonica"),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Tooltip(String::from(loc.msg("options-harmonica-tooltip"))),
        ));
        for (image, name) in previews {
            let is_selected = name == selected;
            r.spawn_empty().apply_scene(harmonica_button_scene(
                image.clone(),
                name.clone(),
                is_selected,
            ));
        }
    });

    commands.entity(parent).add_child(row);
}

/// One harmonica choice button: preview image + name, its dedicated "select
/// this model" click callback (capturing the name), and hover — all inline
/// `on(...)`.
fn harmonica_button_scene(image: Handle<Image>, name: String, is_selected: bool) -> impl Scene {
    let color = if is_selected {
        button::CHOICE_SELECTED
    } else {
        button::color_default()
    };
    let label = name.clone();
    let pick = name.clone();
    bsn! {
        WidgetButton
        TabIndex(0)
        Node {
            flex_direction: {FlexDirection::Column},
            align_items: {AlignItems::Center},
            padding: {UiRect::axes(Val::Px(8.0), Val::Px(6.0))},
            row_gap: {Val::Px(4.0)},
        }
        BackgroundColor({color})
        HarmonicaButton({name})
        on(move |_: On<Activate>, mut selected: ResMut<SelectedHarmonicaModel>| {
            selected.0 = pick.clone();
        })
        on(harm_over)
        on(harm_out)
        Children [
            (
                Node { width: {Val::Px(54.0)}, height: {Val::Px(54.0)} }
                ImageNode { image: {image}, color: {Color::WHITE} }
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            ),
            (
                Text({label})
                TextFont { font_size: {FontSize::Px(16.0)} }
                TextColor({Color::WHITE})
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            )
        ]
    }
}

// ── 3D model previews (render-to-texture) ──────────────────────────────────────

/// Renders a harmonica model's glTF scene to an off-screen texture for the
/// Options UI. Like the note preview, but the model is a multi-mesh scene with
/// its own materials, so it's spawned via `WorldAssetRoot` and shown untinted;
/// `propagate_preview_layers` pushes the render layer onto the scene's children.
fn spawn_harmonica_preview(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    asset_server: &AssetServer,
    model: &str,
    layer: usize,
) -> Handle<Image> {
    let handle = preview_target(images);
    let layers = RenderLayers::layer(layer);

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::NONE),
            order: -1,
            ..default()
        },
        RenderTarget::from(handle.clone()),
        Transform::from_xyz(0.0, 1.6, 4.2).looking_at(Vec3::ZERO, Vec3::Y),
        layers.clone(),
        MenuRoot,
    ));

    // The model scene, posed at a slight angle. Scene children get the render
    // layer from `propagate_preview_layers` (they don't inherit it on spawn).
    commands.spawn((
        WorldAssetRoot(asset_server.load(format!("harmonicas/3d/{model}/harmonica.glb#Scene0"))),
        Transform::from_scale(Vec3::splat(0.1)).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            -0.5,
            0.35,
            0.0,
        )),
        Visibility::default(),
        layers.clone(),
        PreviewSceneLayer(layers.clone()),
        MenuRoot,
    ));

    spawn_preview_light(commands, layers);
    handle
}

/// Allocates a transparent render-target image for a 3D preview.
fn preview_target(images: &mut Assets<Image>) -> Handle<Image> {
    let size = Extent3d {
        width: 128,
        height: 128,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    images.add(image)
}

/// A directional light on `layers` so a preview model is shaded, not flat.
fn spawn_preview_light(commands: &mut Commands, layers: RenderLayers) {
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        layers,
        MenuRoot,
    ));
}

/// Forces each preview scene's render layer onto all of its descendants. glTF
/// scene children spawn a frame or two after the root and don't inherit
/// `RenderLayers`, so without this the preview camera would never see them.
fn propagate_preview_layers(
    mut commands: Commands,
    roots: Query<(Entity, &PreviewSceneLayer)>,
    children: Query<&Children>,
    already_layered: Query<(), With<RenderLayers>>,
) {
    for (root, layer) in &roots {
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                for child in kids {
                    if already_layered.get(*child).is_err() {
                        commands.entity(*child).insert(layer.0.clone());
                    }
                    stack.push(*child);
                }
            }
        }
    }
}

/// Hover highlight for harmonica buttons, never overriding the green selection.
fn harm_over(
    ev: On<Pointer<Over>>,
    selected: Res<SelectedHarmonicaModel>,
    mut buttons: Query<(&HarmonicaButton, &mut BackgroundColor)>,
) {
    if let Ok((btn, mut bg)) = buttons.get_mut(ev.entity)
        && btn.0 != selected.0
    {
        *bg = BackgroundColor(button::CHOICE_HOVER);
    }
}

fn harm_out(
    ev: On<Pointer<Out>>,
    selected: Res<SelectedHarmonicaModel>,
    mut buttons: Query<(&HarmonicaButton, &mut BackgroundColor)>,
) {
    if let Ok((btn, mut bg)) = buttons.get_mut(ev.entity)
        && btn.0 != selected.0
    {
        *bg = BackgroundColor(button::color_default());
    }
}

/// Recolour the harmonica buttons when the selection changes (green = chosen).
fn harmonica_button_visuals(
    selected: Res<SelectedHarmonicaModel>,
    mut buttons: Query<(&HarmonicaButton, &mut BackgroundColor)>,
) {
    if !selected.is_changed() {
        return;
    }
    for (button, mut bg) in &mut buttons {
        bg.0 = if button.0 == selected.0 {
            button::CHOICE_SELECTED
        } else {
            button::color_default()
        };
    }
}

// ── Microphone picker / status banner ───────────────────────────────────────

/// The name of the device actually connected right now, or `None` while
/// [`MicStatus::Failed`]/[`MicStatus::AwaitingPermission`]. Used (rather
/// than the raw `AudioSettings::input_device` preference) so the picker
/// highlights reality — if a saved device went missing and capture fell
/// back to the default, that's what lights up.
fn connected_device_name(status: &MicStatus) -> Option<&str> {
    match status {
        MicStatus::Connected { device_name } => Some(device_name.as_str()),
        MicStatus::Failed { .. } | MicStatus::AwaitingPermission => None,
    }
}

/// Whether the mic warning banner (and the device combobox it stands in
/// for) should be visible — anything other than a successful connection.
fn mic_banner_visible(status: &MicStatus) -> bool {
    !matches!(status, MicStatus::Connected { .. })
}

/// A dismiss-free warning banner, hidden only once the microphone actually
/// connects (see [`mic_banner_visible`]), with a Retry button that re-runs
/// `audio_input::start_capture`.
fn spawn_mic_banner(
    commands: &mut Commands,
    parent: Entity,
    status: &MicStatus,
    loc: &Localization,
) {
    let visible = mic_banner_visible(status);
    let text = mic_banner_text(status);

    let banner = commands
        .spawn((
            Node {
                width: Val::Px(560.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(10.0)),
                display: if visible {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.45, 0.12, 0.12, 0.85)),
            MicBanner,
        ))
        .id();

    commands.entity(banner).with_children(|b| {
        b.spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.85, 0.85)),
            MicBannerText,
        ));
        b.spawn_empty()
            .apply_scene(mic_retry_button_scene(String::from(
                loc.msg("options-mic-retry-tooltip"),
            )));
    });

    commands.entity(parent).add_child(banner);
}

fn mic_banner_text(status: &MicStatus) -> String {
    match status {
        MicStatus::Failed { reason } => format!("No microphone: {reason}"),
        MicStatus::AwaitingPermission => {
            "Waiting for microphone permission — grant it, then retry".to_string()
        }
        MicStatus::Connected { .. } => String::new(),
    }
}

fn mic_retry_button_scene(tooltip: String) -> impl Scene {
    bsn! {
        WidgetButton
        TabIndex(0)
        Node { padding: {UiRect::axes(Val::Px(12.0), Val::Px(6.0))} }
        BackgroundColor({button::color_default()})
        Tooltip({tooltip})
        on(|_: On<Activate>, mut commands: Commands| {
            commands.queue(audio_input::start_capture);
        })
        Children [
            (
                Text({"Retry".to_string()})
                TextFont { font_size: {FontSize::Px(15.0)} }
                TextColor({Color::WHITE})
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            )
        ]
    }
}

/// Show/hide the banner and refresh its reason text when `MicStatus` changes
/// (e.g. after a Retry click or a device-picker selection).
fn update_mic_banner(
    status: Res<MicStatus>,
    mut banners: Query<&mut Node, With<MicBanner>>,
    mut texts: Query<&mut Text, With<MicBannerText>>,
) {
    if !status.is_changed() {
        return;
    }
    let visible = mic_banner_visible(&status);
    for mut node in &mut banners {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    let text = mic_banner_text(&status);
    for mut t in &mut texts {
        **t = text.clone();
    }
}

/// Marks the Options page's microphone combobox root, so [`sync_mic_combobox`]
/// can find it to push `MicStatus` changes into its display — e.g. after
/// Retry reconnects to a different actual device than was last picked, or a
/// saved device disappears and capture silently falls back to the default.
#[derive(Component)]
struct MicCombobox;

/// Wires the shared [`combobox`] widget to the microphone device list:
/// picking an option persists it to `AudioSettings` and reconnects capture
/// immediately.
fn spawn_mic_combobox(
    commands: &mut Commands,
    parent: Entity,
    page_root: Entity,
    loc: &Localization,
    devices: &[String],
    connected: Option<&str>,
) {
    let root = combobox::spawn_combobox(
        commands,
        parent,
        page_root,
        &loc.msg("options-microphone"),
        devices,
        connected.unwrap_or("None"),
        on_mic_selected,
    );
    commands.entity(root).insert((
        MicCombobox,
        Tooltip(String::from(loc.msg("options-microphone-tooltip"))),
    ));
}

fn on_mic_selected(
    ev: On<combobox::ComboboxSelect>,
    mut settings: ResMut<AudioSettings>,
    mut commands: Commands,
) {
    settings.input_device = ev.value.clone();
    commands.queue(audio_input::start_capture);
}

fn sync_mic_combobox(
    status: Res<MicStatus>,
    combo: Query<Entity, With<MicCombobox>>,
    mut values: Query<&mut combobox::ComboboxValue>,
) {
    if !status.is_changed() {
        return;
    }
    let Ok(root) = combo.single() else { return };
    let Ok(mut value) = values.get_mut(root) else {
        return;
    };
    value.0 = connected_device_name(&status).unwrap_or("None").to_string();
}

// ── Dedicated slider callbacks ────────────────────────────────────────────────

fn set_music_volume(ev: On<ValueChange<f32>>, mut settings: ResMut<AudioSettings>) {
    settings.music_volume = ev.value;
}

fn set_metronome_volume(ev: On<ValueChange<f32>>, mut settings: ResMut<AudioSettings>) {
    settings.metronome_volume = ev.value;
}

fn set_input_latency(ev: On<ValueChange<f32>>, mut settings: ResMut<AudioSettings>) {
    settings.input_latency_ms = ev.value.round() as i32;
}

// ── Volume sliders ──────────────────────────────────────────────────────────

/// The shared row shell every Options slider builds on: a 420px row with a
/// 110px label carrying `tooltip`, already attached to `parent` —
/// `spawn_volume_slider`/`spawn_latency_slider`/`spawn_zoom_slider` differ
/// only in the label/tooltip text and, once this returns, which slider
/// track/value-readout they add to the row.
fn spawn_slider_row(commands: &mut Commands, parent: Entity, label: &str, tooltip: &str) -> Entity {
    let row = commands
        .spawn(Node {
            width: Val::Px(420.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(14.0),
            ..default()
        })
        .id();
    commands.entity(row).with_children(|r| {
        r.spawn((
            Node {
                width: Val::Px(110.0),
                ..default()
            },
            Text::new(label.to_string()),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    commands.entity(row).insert(Tooltip(tooltip.to_string()));
    commands.entity(parent).add_child(row);
    row
}

/// The shared trailing value-readout `spawn_volume_slider`/
/// `spawn_latency_slider` both append after their track — a 50px right-hand
/// label tagged with whichever marker component that caller's own sync
/// system reads (`SliderValueLabel`/`LatencySliderLabel`).
fn spawn_slider_value_label(
    commands: &mut Commands,
    row: Entity,
    text: String,
    marker: impl Component,
) {
    commands.entity(row).with_children(|r| {
        r.spawn((
            Node {
                width: Val::Px(50.0),
                ..default()
            },
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.7)),
            marker,
        ));
    });
}

fn spawn_volume_slider<M: 'static>(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    tooltip: &str,
    kind: VolumeSlider,
    value: f32,
    on_change: impl IntoObserverSystem<ValueChange<f32>, (), M> + Clone + Sync + 'static,
) {
    let row = spawn_slider_row(commands, parent, label, tooltip);

    // SliderRange/SliderStep have no Default, so they can't be bsn! patches —
    // insert them after the scene is spawned.
    let track = commands
        .spawn_scene(volume_slider_scene(kind, value, on_change))
        .insert((SliderRange::new(0.0, 1.0), SliderStep(0.01)))
        .id();
    commands.entity(row).add_child(track);

    spawn_slider_value_label(
        commands,
        row,
        format!("{:.0}%", value * 100.0),
        SliderValueLabel(kind),
    );
}

/// The volume slider track itself: a `bsn!` `Slider` with its fill, wired to the
/// given value-change callback inline via `on(...)`.
fn volume_slider_scene<M: 'static>(
    kind: VolumeSlider,
    value: f32,
    on_change: impl IntoObserverSystem<ValueChange<f32>, (), M> + Clone + Sync + 'static,
) -> impl Scene {
    bsn! {
        Slider { track_click: {TrackClick::Snap} }
        TabIndex(0)
        SliderValue({value})
        Node { width: {Val::Px(220.0)}, height: {Val::Px(14.0)} }
        BackgroundColor({TRACK_BG})
        on(on_change)
        Children [
            (
                Node { width: {Val::Percent(value * 100.0)}, height: {Val::Percent(100.0)} }
                BackgroundColor({Color::srgb(0.35, 0.75, 1.0)})
                SliderFill({kind})
                // Don't let the fill steal the slider's pointer events.
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            )
        ]
    }
}

// ── Input-latency slider ──────────────────────────────────────────────────────

const LATENCY_MAX_MS: i32 = 200;

/// One labelled slider row for the mic input-latency offset.
/// The track maps 0–200 ms linearly; the label shows "Xms".
fn spawn_latency_slider(
    commands: &mut Commands,
    parent: Entity,
    value_ms: i32,
    loc: &Localization,
) {
    let frac = (value_ms as f32 / LATENCY_MAX_MS as f32).clamp(0.0, 1.0);
    let label = String::from(loc.msg("options-input-lag"));
    let tooltip = String::from(loc.msg("options-input-lag-tooltip"));
    let row = spawn_slider_row(commands, parent, &label, &tooltip);

    let track = commands
        .spawn_scene(latency_slider_scene(value_ms as f32, frac))
        .insert((
            SliderRange::new(0.0, LATENCY_MAX_MS as f32),
            SliderStep(1.0),
        ))
        .id();
    commands.entity(row).add_child(track);

    spawn_slider_value_label(commands, row, format!("{value_ms}ms"), LatencySliderLabel);
}

/// The latency slider track: a `bsn!` `Slider` + fill, wired to `set_input_latency`.
fn latency_slider_scene(value: f32, frac: f32) -> impl Scene {
    bsn! {
        Slider { track_click: {TrackClick::Snap} }
        TabIndex(0)
        SliderValue({value})
        Node { width: {Val::Px(220.0)}, height: {Val::Px(14.0)} }
        BackgroundColor({TRACK_BG})
        LatencySlider
        on(set_input_latency)
        Children [
            (
                Node { width: {Val::Percent(frac * 100.0)}, height: {Val::Percent(100.0)} }
                BackgroundColor({Color::srgb(0.80, 0.55, 0.25)})
                LatencySliderFill
                Pickable { should_block_lower: {false}, is_hoverable: {false} }
            )
        ]
    }
}

/// Mirror `input_latency_ms` onto the fill bar and label.
fn update_latency_slider(
    settings: Res<AudioSettings>,
    mut fills: Query<&mut Node, With<LatencySliderFill>>,
    mut labels: Query<&mut Text, With<LatencySliderLabel>>,
) {
    if !settings.is_changed() {
        return;
    }
    let frac = (settings.input_latency_ms as f32 / LATENCY_MAX_MS as f32).clamp(0.0, 1.0);
    for mut node in &mut fills {
        node.width = Val::Percent(frac * 100.0);
    }
    for mut text in &mut labels {
        text.0 = format!("{}ms", settings.input_latency_ms);
    }
}

/// Mirror the current levels onto the slider fills and percentage readouts.
fn update_sliders(
    settings: Res<AudioSettings>,
    mut fills: Query<(&mut Node, &SliderFill)>,
    mut labels: Query<(&mut Text, &SliderValueLabel)>,
) {
    for (mut node, fill) in &mut fills {
        node.width = Val::Percent(audio_level(&settings, fill.0) * 100.0);
    }
    for (mut text, label) in &mut labels {
        text.0 = format!("{:.0}%", audio_level(&settings, label.0) * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_label_shows_the_rounded_percent() {
        let loc = Localization::default();
        assert_eq!(zoom_label_text(&loc, 1.0), "options-zoom-label");
    }

    #[test]
    fn zoom_fraction_spans_min_to_max() {
        use crate::dialogs::ui_scale::{MAX_SCALE, MIN_SCALE};
        assert_eq!(zoom_fraction(MIN_SCALE), 0.0);
        assert_eq!(zoom_fraction(MAX_SCALE), 1.0);
        assert!((zoom_fraction((MIN_SCALE + MAX_SCALE) / 2.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn zoom_fraction_clamps_outside_the_range() {
        use crate::dialogs::ui_scale::{MAX_SCALE, MIN_SCALE};
        assert_eq!(zoom_fraction(MIN_SCALE - 5.0), 0.0);
        assert_eq!(zoom_fraction(MAX_SCALE + 5.0), 1.0);
    }

    #[test]
    fn mic_banner_hidden_only_once_connected() {
        assert!(!mic_banner_visible(&MicStatus::Connected {
            device_name: "Mic".into(),
        }));
        assert!(mic_banner_visible(&MicStatus::Failed {
            reason: "no device".into(),
        }));
        assert!(mic_banner_visible(&MicStatus::AwaitingPermission));
    }

    #[test]
    fn connected_device_name_is_none_unless_connected() {
        assert_eq!(
            connected_device_name(&MicStatus::Connected {
                device_name: "USB Mic".into(),
            }),
            Some("USB Mic")
        );
        assert_eq!(
            connected_device_name(&MicStatus::Failed {
                reason: "no device".into(),
            }),
            None
        );
        assert_eq!(connected_device_name(&MicStatus::AwaitingPermission), None);
    }

    #[test]
    fn mic_banner_text_is_distinct_per_status() {
        assert_eq!(
            mic_banner_text(&MicStatus::Connected {
                device_name: "Mic".into(),
            }),
            ""
        );
        assert!(
            mic_banner_text(&MicStatus::Failed {
                reason: "no device".into(),
            })
            .contains("no device")
        );
        assert_ne!(
            mic_banner_text(&MicStatus::AwaitingPermission),
            mic_banner_text(&MicStatus::Failed {
                reason: "no device".into(),
            }),
            "awaiting-permission needs its own message, not the generic failure one"
        );
    }
}
