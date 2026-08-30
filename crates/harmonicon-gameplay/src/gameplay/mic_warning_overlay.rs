// SPDX-License-Identifier: MIT

//! Says so, during play, when the microphone isn't working.
//!
//! A dead mic and a player who simply isn't hitting anything look identical
//! from the inside: notes scroll past, nothing scores, the combo stays at
//! zero. The distinction was reported only on the Options page — somewhere
//! the confused player has no reason to go, since from their seat the game
//! isn't broken, they are.
//!
//! Spawned for every mode under `AppState::Playing` (Play 2D, Play 3D and
//! Jam Session alike): Jam Session isn't scored, but it still feeds back
//! against what it hears, so a deaf one is just as misleading.

use bevy::prelude::*;
use bevy_fluent::Localization;

use harmonicon_app::app::AppState;
use harmonicon_audio::audio_input::MicStatus;
use harmonicon_platform::localization::LocalizationExt;

use super::GameplayRoot;

/// The warning's text node.
#[derive(Component, Default, Clone)]
pub struct MicWarningLabel;

/// The warning's outermost node. Visibility is toggled here rather than on
/// [`MicWarningLabel`], because the label sits inside a padded, coloured
/// panel — hiding just the text would leave an empty red box over the
/// highway.
#[derive(Component, Default, Clone)]
pub struct MicWarningRoot;

/// Spawns the (initially hidden) warning. Tagged `GameplayRoot` so it's torn
/// down with the rest of the scene.
///
/// Sits immediately *below* the song-progress bar, offset by that bar's own
/// [`song_progress_overlay::BAR_HEIGHT`] rather than a copied number, so the
/// two can't drift apart. It has to clear that bar rather than share the
/// top edge with it: the bar paints at `GlobalZIndex(250)` — above the pause
/// overlay, deliberately — so anything overlapping it is painted over,
/// which is exactly what a first attempt at `top: 8px` did (the warning
/// rendered *behind* the note-lane markers and was unreadable).
///
/// Its own z sits above the mode scenes (1) and their HUD panels (100) but
/// below the pause overlay (200), so pausing still covers it — a paused
/// player is already being told what to do.
///
/// A system registered on `OnEnter(AppState::Playing)` rather than a helper
/// called from `gameplay_2d`/`gameplay_3d`'s own setup, which is how
/// `spawn_wait_freeze_prompt` is wired: those two run only in their own
/// modes, and Jam Session needs this warning just as much.
pub fn setup_mic_warning(mut commands: Commands) {
    commands
        .spawn_scene(bsn! {
            Node {
                position_type: {PositionType::Absolute},
                top: {Val::Px(super::song_progress_overlay::BAR_HEIGHT + 8.0)},
                width: {Val::Percent(100.0)},
                flex_direction: {FlexDirection::Column},
                align_items: {AlignItems::Center},
            }
            GlobalZIndex(150)
            GameplayRoot
            MicWarningRoot
            Children [
                (
                    Node {
                        padding: {UiRect::axes(Val::Px(16.0), Val::Px(8.0))},
                        border_radius: {BorderRadius::all(Val::Px(6.0))},
                    }
                    BackgroundColor({Color::srgba(0.35, 0.05, 0.05, 0.92)})
                    Children [
                        (
                            Text({""})
                            TextFont { font_size: {FontSize::Px(20.0)} }
                            TextColor({Color::srgb(1.0, 0.85, 0.85)})
                            MicWarningLabel
                        )
                    ]
                )
            ]
        })
        .insert(Visibility::Hidden);
}

/// The localized message for a status, or `None` when the mic is fine.
pub fn mic_warning_text(status: &MicStatus, loc: &Localization) -> Option<String> {
    mic_warning_key(status).map(|key| String::from(loc.msg(key)))
}

/// Which Fluent key a status warrants, or `None` when the mic is fine.
///
/// The pure half of [`mic_warning_text`], split out so that "does each
/// problem get its own message" is testable — the text itself needs a
/// `Localization`, which a unit test has no cheap way to build. The two are
/// not interchangeable: one says "grant a permission", the other "your
/// device is broken or busy".
pub fn mic_warning_key(status: &MicStatus) -> Option<&'static str> {
    match status {
        MicStatus::Connected { .. } => None,
        MicStatus::AwaitingPermission => Some("mic-warning-permission"),
        MicStatus::Failed { .. } => Some("mic-warning-failed"),
    }
}

/// Shows or hides the warning as [`MicStatus`] changes.
///
/// The deliberately-omitted detail is the `Failed { reason }` string: it's a
/// raw cpal error ("Device not available", "The requested device is no
/// longer available"), untranslated and meaningless mid-song. The Options
/// page is where the specifics belong; this is a signpost pointing at it.
fn sync_mic_warning(
    status: Res<MicStatus>,
    loc: Res<Localization>,
    mut labels: Query<&mut Text, With<MicWarningLabel>>,
    mut roots: Query<&mut Visibility, With<MicWarningRoot>>,
) {
    if !status.is_changed() {
        return;
    }
    let message = mic_warning_text(&status, &loc);
    if let Some(message) = &message {
        for mut text in &mut labels {
            *text = Text::new(message.clone());
        }
    }
    for mut vis in &mut roots {
        *vis = if message.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub struct MicWarningPlugin;

impl Plugin for MicWarningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_mic_warning.run_if(in_state(AppState::Playing)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_working_microphone_warns_about_nothing() {
        assert_eq!(
            mic_warning_key(&MicStatus::Connected {
                device_name: "Some Mic".into(),
            }),
            None
        );
    }

    #[test]
    fn each_kind_of_trouble_gets_its_own_message() {
        // "grant a permission" and "your device is broken" are different
        // problems with different fixes; one generic message for both would
        // send an Android player hunting through Options for nothing.
        assert_ne!(
            mic_warning_key(&MicStatus::AwaitingPermission),
            mic_warning_key(&MicStatus::Failed {
                reason: "Device not available".into(),
            })
        );
    }

    #[test]
    fn the_in_play_warning_never_shows_the_raw_device_error() {
        // The reason is an untranslated cpal string, meaningless mid-song —
        // Options is where the specifics belong. Keying off `&'static str`
        // makes that structural rather than a habit: there is nowhere to
        // interpolate it.
        let key = mic_warning_key(&MicStatus::Failed {
            reason: "Device not available".into(),
        });
        assert_eq!(key, Some("mic-warning-failed"));
    }
}
