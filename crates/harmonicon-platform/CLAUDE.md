# harmonicon-platform

What the game needs from the machine it runs on: asset discovery,
localization, persisted settings, the visual theme, and the narrow-window
breakpoint.

Nothing here knows what a song, a note or a screen is.

Project-wide rules (workspace layering, localization, testing style,
commit conventions) are in the root `CLAUDE.md` — this file is only what's
load-bearing about *this* crate.

## Architecture (load-bearing facts)

- **Asset sources:** bundled `assets/` plus an `external://` source mapped to
  `~/Harmonicon` (registered in `main.rs` before DefaultPlugins). When
  loading siblings of an asset, propagate its source or external songs
  silently resolve against the bundled tree (see comment in `song/loader.rs`).
  - **`~/Harmonicon` is watched live**, not just scanned once at Startup:
    `assets_management::watch` starts one recursive `notify-debouncer-full`
    watcher on it (our own direct dependency, no-op if the folder doesn't
    exist — most players never create it), debounces bursts of filesystem
    events, and fires one generic `ExternalFolderChanged{top_level_dirs}`
    message per batch naming which immediate subfolders (`songs`, `themes`,
    `lessons`, ...) something changed under
    (`watch::changed_top_level_dirs`) — `watch.rs` itself stays agnostic of
    what any of those subfolders *mean* (see "dependencies point downward"
    in `docs/physical_design_plan.md`). `assets_management::mod.rs`'s own
    `rescan_on_external_change` consumes that message for the two kinds
    this module owns (`songs`/`themes`), re-running `scan_all_songs`/
    `scan_ui_themes`; `lessons::catalog` has its own sibling consumer for
    `lessons` (see the Lessons bullet below) — one small
    `lessons`-depends-on-`assets_management` edge rather than the reverse.
    Every scan function fully replaces its resource's contents rather than
    appending, so each is safe to call again at runtime (`scan_all_songs`
    clears `AvailableSongs` first — it didn't always; `scan_ui_themes`/
    `scan_lessons` already assigned wholesale). A successful live rescan
    also fires its own specific `SongsRescanned`/`ThemesRescanned`/
    `LessonsRescanned` — a message, not a bare `is_changed()` poll, because
    the menu pages that consume them only run their consuming system while
    open, so their own change-detection tick would otherwise read
    stale-as-changed on every re-entry rather than only on a genuine live
    drop-in. Deliberately **not** built on `bevy::asset::io::file::
    FileWatcher`/Bevy's own asset-hot-reload path: that path only reloads
    already-loaded `Handle`s (useless for content that was never loaded to
    begin with), and whether *any* source watches at all is one global
    `AssetPlugin::watch_for_changes_override` flag applied to every
    registered source uniformly — turning it on for `external://` would
    also enable asset hot-reloading for the bundled `assets/` tree in
    shipped builds, which is exactly the `--features dev`-only behavior
    this file's Commands section says never to ship.

- **Settings:** figment-layered `<config>/harmonicon/settings.json`
  (`settings.rs`); saves are debounced (`PendingSave`, 0.5 s) with a flush
  on `AppExit` — route new persisted fields through that path.

- **Responsive/compact layout for narrow windows** (`harmonicon-platform`'s `responsive.rs`).
  `CompactLayout` (a `Resource`) is derived every frame from the primary
  window's width divided by `UiScale` (the same effective-width math
  `song_editor::interaction`'s own scroll-clamping already used) crossing a
  single shared `COMPACT_BREAKPOINT_PX` (900.0) — one definition of
  "compact" reused everywhere, rather than each screen picking its own
  threshold. Deliberately **not live-reactive**: `gameplay_2d::setup`/
  `gameplay_3d::setup` and `song_editor::ui::setup` each read
  `Res<CompactLayout>` once, at `OnEnter(AppState::Playing)`/
  `OnEnter(AppState::SongEditor2)`, and branch what they spawn — a resize
  *during* a song or edit session doesn't retroactively reflow it. Live-
  reflowing an already-spawned scene would need despawn/respawn logic on
  par with a second setup system, for a scenario that matters far less than
  "the screen you land on already fits" — extend this only if that turns
  out to be wrong.
  - **Play 2D/3D compact**: the note highway (2D) / 3D scene stays, plus a
    minimal score/combo/feedback readout; everything else gameplay's HUD
    normally shows — song info, phrase banner, tab ribbon, the 12-bar
    grid, metronome, technique legend, and the `music_score` notation
    staff — is skipped entirely rather than shrunk, since none of it is
    essential to actually playing. `gameplay_3d::setup` bundles `theme`/
    `loc`/`bravura`/`compact` into a new `HudContext` `SystemParam`
    (mirroring the file's own pre-existing `NoteBuildState`) purely
    because plain individual params would have put it one over Bevy's
    function-system arity limit — nothing about the four belonging
    together otherwise.
  - **Song Editor compact**: `meta_form::spawn_meta_form`'s three
    side-by-side columns (content-kind/harmonica/snap fields, the rest of
    the fields + MIDI row, the color legend) stack vertically instead —
    the tightest rigid constraint found anywhere in the app (≈1122px
    before clipping, no wrap, only vertical scrolling available). The top
    transport strip (`mod_panel.rs`) also gained `flex_wrap`, unconditionally
    (not gated on `CompactLayout` at all — wrapping only engages once
    content overflows, so it's a strict improvement on wide screens too).
    The note grid needed no changes — it already self-scrolls horizontally
    regardless of window width (`grid.rs`'s own `visible_beats`/
    `GridScrollTrack`).
  - **Menu pages are out of scope** — they already handle overflow via
    `menu::scene::spawn_menu_root`'s scroll area, so they were judged
    reasonably small-screen-safe already.
