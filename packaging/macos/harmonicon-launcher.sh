#!/bin/sh
# macOS .app bundle launcher for Harmonicon — the `CFBundleExecutable`
# LaunchServices actually runs, not the real game binary (that's installed
# alongside it as `harmonicon-bin`).
#
# Several of this project's own startup scans (assets_management, theme.rs,
# lessons::catalog, dialogs::file_dialog, menu::pages::credits) read
# `assets/...` as a path relative to the current working directory, not
# through Bevy's own AssetServer — same reason
# packaging/flatpak/harmonicon.sh needs this same `cd`. Finder/LaunchServices
# launches a `.app` with an arbitrary CWD (typically the user's home
# directory, not the bundle's own folder), so without this, every one of
# those scans silently comes up empty — no songs, no themes, no lessons —
# the moment the app is double-clicked, even though the very same bundle
# works fine when the real binary is run directly from a shell already
# sitting in this directory.
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR" || exit 1
exec "$DIR/harmonicon-bin" "$@"
