// SPDX-License-Identifier: MIT

//! Keyboard focus navigation: registers `bevy_input_focus`'s Tab/Shift+Tab
//! cycling (not part of `DefaultPlugins`, unlike `InputFocusPlugin`/
//! `InputDispatchPlugin`, which already are) and bridges its keyboard
//! activation to this codebase's click handlers.
//!
//! `bevy_ui_widgets::Button` already turns a focused Enter/Space into an
//! [`Activate`] event via its own `button_on_key_event` observer (part of
//! `UiWidgetsPlugins`, already included by `DefaultPlugins` under the
//! `bevy_ui_widgets` feature) — the same event a mouse click produces
//! internally. But every click handler in this codebase (~130 call sites
//! across menu/dialogs/song_editor/gameplay) is wired to `Pointer<Click>`,
//! not `Activate`. Rather than retype every one of them, [`bridge_activate_to_click`]
//! re-triggers a `Pointer<Click>` on the activated entity so existing
//! handlers fire unchanged for keyboard activation too. Every screen this
//! applies to marks its focusable widgets with `TabIndex(0)` (ties within
//! one `TabGroup` break by spawn order, so no hand-authored ordering is
//! needed — see `bevy_input_focus::tab_navigation`'s own doc comment) and
//! its root with a `TabGroup`.

use std::time::Duration;

use bevy::camera::{ManualTextureViewHandle, NormalizedRenderTarget};
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::picking::backend::HitData;
use bevy::picking::events::{Click, Pointer};
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

/// Outline color for whichever control currently has keyboard focus —
/// distinct from this UI's existing hover (`dialogs::button::CHOICE_HOVER`,
/// a dark blue-grey) and selected (`dialogs::button::CHOICE_SELECTED`, a
/// green) tints, so it reads as its own kind of feedback.
const FOCUS_OUTLINE_COLOR: Color = Color::srgb(0.95, 0.80, 0.20);
const FOCUS_OUTLINE_WIDTH: f32 = 2.0;
const FOCUS_OUTLINE_OFFSET: f32 = 2.0;

/// A synthetic pointer location carrying no real render target — safe here
/// because nothing downstream reads pointer position/target from a
/// bridged click (every in-scope `Pointer<Click>` handler reads only
/// `.entity`). Mirrors bevy_picking's own `STUB_LOCATION` test pattern
/// (`bevy_picking::events::tests`), which uses the same
/// `TextureView(ManualTextureViewHandle(_))` placeholder for exactly this
/// reason.
fn synthetic_location() -> Location {
    Location {
        target: NormalizedRenderTarget::TextureView(ManualTextureViewHandle(0)),
        position: Vec2::ZERO,
    }
}

/// Bridges keyboard/gamepad activation to this codebase's click handlers
/// — see the module doc comment for why. Can't loop back into itself via
/// `bevy_ui_widgets::button_on_pointer_click`: that observer only
/// re-triggers `Activate` when the clicked entity carries `Pressed`
/// (set by a real pointer-down), which a synthetic click never has.
fn bridge_activate_to_click(ev: On<Activate>, mut commands: Commands) {
    commands.trigger(Pointer::new(
        PointerId::Mouse,
        synthetic_location(),
        Click {
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: Duration::ZERO,
            count: 1,
        },
        ev.entity,
    ));
}

/// Paints a visible outline on whichever entity currently has keyboard
/// focus, following [`InputFocus`]/[`InputFocusVisible`] (the latter is
/// kept in sync automatically by `TabNavigationPlugin` itself — hidden on
/// a mouse click, shown again on Tab — nothing to hand-roll here). Uses
/// `Outline`, not `BorderColor`: it draws outside the box with no layout
/// impact, so focusable widgets don't need to reserve border space up
/// front just to support this. Only touches the previous/current focused
/// entity, not a whole-UI scan; `try_insert` since either can have
/// despawned since last frame (e.g. a page navigation).
fn update_focus_ring(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    mut last_painted: Local<Option<Entity>>,
    mut commands: Commands,
) {
    if !focus.is_changed() && !visible.is_changed() {
        return;
    }
    if let Some(prev) = last_painted.take()
        && Some(prev) != focus.get()
    {
        commands.entity(prev).try_insert(Outline {
            width: Val::ZERO,
            offset: Val::ZERO,
            color: Color::NONE,
        });
    }
    let Some(current) = focus.get().filter(|_| visible.0) else {
        return;
    };
    commands.entity(current).try_insert(Outline {
        width: Val::Px(FOCUS_OUTLINE_WIDTH),
        offset: Val::Px(FOCUS_OUTLINE_OFFSET),
        color: FOCUS_OUTLINE_COLOR,
    });
    *last_painted = Some(current);
}

pub struct KeyboardNavPlugin;

impl Plugin for KeyboardNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .add_observer(bridge_activate_to_click)
            .add_systems(Update, update_focus_ring);
    }
}
