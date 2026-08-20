// SPDX-License-Identifier: MIT

//! Shared full-screen page chrome: the header row (title/subtitle on the
//! left, a Back control on the right) that every menu page and every
//! full-screen non-menu `AppState` builds its top bar from.
//!
//! Lives here rather than in `menu::scene` because it is not menu-specific:
//! `gameplay::bending_trainer` is a full-screen `AppState` of its own and
//! composes the same header directly. `gameplay` sits below `menu` and may
//! not import upward (`docs/physical_design_plan.md` rule 2), so the shared
//! part belongs in `dialogs` with the rest of the reusable widgets.

use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::button;
use super::tooltip::Tooltip;

pub fn heading_scene(text: String, size: f32, color: Color) -> impl Scene {
    bsn! {
        Text({text})
        TextFont { font_size: {FontSize::Px(size)} }
        TextColor({color})
    }
}

/// The header row: a full-width `Row` holding the title/subtitle column
/// (`flex_grow: 1.0` — the "horizontal-stretch" that pushes a trailing
/// [`spawn_back_button`] to the row's far edge) plus whatever a page adds
/// via that function. `flex_shrink: 0.0` keeps it at its natural height
/// even if `body`'s content overflows. `pub(crate)`, alongside
/// [`title_column_scene`], so a full-screen `AppState` that isn't an
/// ordinary menu page (e.g. `gameplay::bending_trainer`) can still build
/// the same title-top-left/back-top-right header without going through
/// [`spawn_menu_root`]'s whole page shape (background image, scroll area,
/// `MenuRoot` cleanup tag) — see that module's own header for how it's
/// composed directly with [`spawn_back_button`].
pub fn header_scene() -> impl Scene {
    bsn! {
        Node {
            width: {Val::Percent(100.0)},
            flex_direction: {FlexDirection::Row},
            align_items: {AlignItems::Center},
            column_gap: {Val::Px(16.0)},
            padding: {UiRect::axes(Val::Px(32.0), Val::Px(20.0))},
            flex_shrink: {0.0_f32},
        }
    }
}

pub fn title_column_scene(title: String) -> impl Scene {
    bsn! {
        Node {
            flex_direction: {FlexDirection::Column},
            flex_grow: {1.0_f32},
            row_gap: {Val::Px(6.0)},
        }
        Children [ heading_scene(title, 52.0, Color::WHITE) ]
    }
}

/// Spawn a page's Back control in its `header` (the second value returned
/// by [`spawn_menu_root`]) — an icon-only button (`dialogs::button::icon`)
/// with `tooltip` attached as a [`Tooltip`], since there's no visible label
/// to explain the glyph. Sitting after the header's `flex_grow: 1.0` title
/// column, it lands at the row's trailing edge for free. A page with no
/// back target (Main Menu) simply never calls this.
pub fn spawn_back_button<M: 'static>(
    commands: &mut Commands,
    header: Entity,
    tooltip: &str,
    on_click: impl IntoObserverSystem<Activate, (), M> + Clone + Sync + 'static,
) -> Entity {
    let e = commands
        .spawn_scene(button::icon("\u{2190}", on_click))
        .insert(Tooltip(tooltip.to_string()))
        .id();
    commands.entity(header).add_child(e);
    e
}
