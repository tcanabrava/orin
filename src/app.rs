// SPDX-License-Identifier: MIT

//! App-wide vocabulary: the top-level state machine ([`AppState`]), the
//! gameplay mode selector ([`GameplayMode`]), the currently-selected
//! song/artist, and the cross-state `ReturnTo*` routing flags.
//!
//! Pure data plus the trivial run conditions over it — every feature
//! (gameplay, song editor, spectrogram, profile, menu) shares this level;
//! nothing here imports a feature.

use bevy::prelude::*;

use harmonicon_core::chart::Scale;
use harmonicon_core::harmonica::Progression;
use harmonicon_song::song::SongManifest;

// ── App-level states ──────────────────────────────────────────────────────────

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Startup,
    Menu,
    SongLoading,
    Playing,
    /// Post-song results / statistics screen.
    Results,
    /// Latency calibration screen (outside the menu sub-state hierarchy).
    Calibration,
    /// Credits screen with scrolling text and 3D harmonica background.
    Credits,
    /// Song authoring tool, launched from the main menu.
    SongEditor2,
    /// Standalone bending practice: harmonica bend diagram + metronome, with a
    /// directly pickable key and adjustable tempo (no song).
    BendingTrainer,
}

#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub enum GameplayMode {
    #[default]
    Play2D,
    Play3D,
    /// Free-play: the 12-bar chart + metronome, no falling notes.
    JamSession,
}

/// The 12-bar variant Jam Session's grid/hole-map/(for a generated jam)
/// backing audio all follow — see `song::harmonica::Progression`. Only ever
/// anything but `Standard` for a "Generate Jam" session (`menu::
/// jam_generate` sets it explicitly on Start); the real-song "Jam Session"
/// button resets it to `Standard` so a previous generated jam's pick can't
/// leak into a real song (which always plays its own actual chords,
/// regardless of this resource — see `twelve_bar_blues_overlay::update_bar`).
#[derive(Resource, Default)]
pub struct JamProgression(pub Progression);

/// The scale Jam Session's live hole-map feedback (`jam::session::
/// JamHoleGuide`) judges played notes against — see `song::chart::Scale`.
/// Defaults to `FirstPosition` (the blues hexatonic — unchanged Jam
/// Session behavior before this resource existed). Set explicitly by
/// "Generate Jam" (`menu::jam_generate`) and by a jam-based lesson's
/// `scale` manifest field (`menu::pages::lessons::parse_scale`); the
/// real-song "Jam Session" button resets it to `FirstPosition`, mirroring
/// `JamProgression`'s own reset — though a real song's own declared
/// `Harmonica::scale()` (if it sets one) still wins over this resource,
/// see `jam::session::build_hole_guide`'s caller.
#[derive(Resource, Default)]
pub struct JamScale(pub Scale);

/// Whether Jam Session should periodically call a new position (cycling
/// `JamScale` through First/Second/Third position) — see
/// `jam::position_guide`. Only ever `true` for a jam-based lesson that opts
/// in via its manifest's `position_cycle` field
/// (`menu::pages::lessons::setup_lesson_reader`'s Start handler); the
/// real-song "Jam Session" button resets it to `false`, mirroring
/// `JamProgression`/`JamScale`'s own reset, so a previous lesson's cycling
/// can't leak into an ordinary jam.
#[derive(Resource, Default)]
pub struct JamPositionCycle(pub bool);

/// Present while a generated-backing jam is in flight (from the "Start Jam"
/// button through `Playing`, including any Restart). Its presence — checked
/// by both `menu::route_menu_entry` and `gameplay::pause_menu::on_restart`
/// — tells those call sites this `SelectedSong` was built by
/// [`build_generated_manifest`] via `Assets::add` rather than loaded through
/// the `AssetServer`, so it has no tracked `LoadState`: both routes skip
/// `AppState::SongLoading` and go straight to `Playing`. Removed on
/// returning to the menu, same end-of-life point `LessonContext` uses.
#[derive(Resource)]
pub struct GeneratedJamSession;

/// Set while the guided tutorial tour (`menu::pages::tutorial`) is driving
/// the app automatically. Every screen the tour passes through
/// (`gameplay::pause_menu`, the Bending Trainer, the Song Editor's grid
/// keys) gates its own Escape/pause handling on this, so the tour's
/// click-blocking overlay isn't the only thing keeping the player from
/// steering it off course — "Skip Tutorial" is the one deliberate way out.
///
/// The tour's real state (`TutorialTour`: step, timer, return page) stays in
/// `menu`, which is the only writer; this flag is *derived* from it every
/// frame by `menu::pages::tutorial::sync_tour_active`. It lives down here
/// because `gameplay` and `song_editor` sit below `menu` and may not import
/// upward (`docs/physical_design_plan.md` rule 2).
#[derive(Resource, Default)]
pub struct TourActive(pub bool);

/// True while a guided tour is running — see [`TourActive`].
pub fn tour_active(tour: Res<TourActive>) -> bool {
    tour.0
}

// ── Selection resources ───────────────────────────────────────────────────────

#[derive(Resource)]
pub struct SelectedSong(pub Handle<SongManifest>);

#[derive(Resource, Default)]
pub struct SelectedArtist(pub String);

// ── Cross-state routing flags ─────────────────────────────────────────────────
//
// Crossing an `AppState` boundary back into `Menu` can't set
// `NextState<MenuPage>` directly — it loses to the substate machinery
// resetting to its own default first — so exits set one of these flags and
// `menu::route_menu_entry` consumes it on arrival.

/// Set to `true` by the pause menu's "Quit Song" button so that re-entering
/// `AppState::Menu` lands on the song list rather than the main menu.
#[derive(Resource, Default)]
pub struct ReturnToSongList(pub bool);

/// Set to `true` by the calibration screen so that returning to `AppState::Menu`
/// lands on the Options page (where the Input lag slider lives).
#[derive(Resource, Default)]
pub struct ReturnToOptions(pub bool);

/// Set to `true` by the Song Editor (`AppState::SongEditor2`) on every exit
/// path so that returning to `AppState::Menu` lands on the Play page (where
/// "Create Song" lives) rather than the substate's own default of Main.
#[derive(Resource, Default)]
pub struct ReturnToPlay(pub bool);

/// Set to `true` by the Credits screen (`AppState::Credits`) on every exit
/// path so that returning to `AppState::Menu` lands on the Help/About page
/// (where "Credits" lives) rather than the substate's own default of Main.
#[derive(Resource, Default)]
pub struct ReturnToHelpAbout(pub bool);
