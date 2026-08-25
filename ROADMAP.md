# Roadmap

Where Harmonicon goes from `0.1.0`. The theme of the project is **teaching
blues and jazz harmonica through play** — every milestone should move a
self-taught player further than a YouTube tutorial would.

Near-term bug/cleanup work lives in `TODO.md`; the execution order and
implementation notes live in `PLAN.md`.

**0.2 "Trustworthy" and 0.3 "Practice" are fully shipped** (see `PLAN.md`'s
Shipped section). No release tags have been cut for either yet
(`Cargo.toml` still says `0.1.0`; the newest tag is `v0.0.1.1`); cut one
per phase once its exit criteria pass.

## 0.4 — "Blues school" (curriculum & jam) — in progress

Adaptive difficulty, the jam-session position/scale overlays, the lessons
engine with its first content pass, generated backing tracks, selectable
jam progressions and playing positions (1st/2nd/3rd, picking the matching
cross-harp key), freeform (unscored) call-and-response practice in Jam
Session (`jam::call_response`), and lessons content wave 2 (harmonica
basics, bar-counting drills, train-rhythm chugging, and the
blues-vocabulary Unit 3 — licks via call-and-response, chord-tone/
minor-blues/phrase-discipline improvisation) are all done — see
`docs/lessons_plan.md`. Open:

- **Backing track variety, remainder**: a set of recorded loops per style
  (shuffle, slow blues, swing) as a richer alternative/addition to the
  generated bass.

## 0.5 — "Content" (authoring & ecosystem)

- **Song editor maturity — done.** The happy-path authoring round-trip
  (record → edit → validate → play, without touching JSON) is functionally
  complete (Record/Edit/Play modes, MIDI import with key suggestion, a real
  multi-point tempo map, lesson authoring alongside plain songs), and the
  workflow/UX pass that followed (2026-07-27: undo/redo, metronome/
  count-in, note audition, save/validation feedback, and a swing/triplet-
  aware grid snap — see `PLAN.md`'s Shipped section) closed out every item
  it found.
- Downloadable song packs / community sharing for the `~/Harmonicon`
  external-source folder. Live auto-refresh of that folder (songs, themes,
  and lessons) is done — see `PLAN.md`. The actual packaging/download/
  hosting mechanism for community song packs is still open — a product
  decision (where packs are hosted, how they're verified) rather than a
  small code task.
- More bundled public-domain songs across all four difficulties (see
  `TODO.md`'s content-gap item).
- Per-technique playback effects (pitch-bend/vibrato/wah DSP driven by chart
  modifiers) — a `fx_mapping`-style chart field was removed as an unbuilt
  stub; reintroduce something like it if/when this gets built.

## 0.6 — "Jazz" (advanced curriculum)

A distinct milestone from 0.4's blues curriculum — bigger scope, split out
because it needs its own content and, likely, chromatic-harmonica-specific
teaching. The lesson-level breakdown (swing-feel drills, ii–V–I arpeggio
lessons, chromatic slide curriculum, jazz-blues form) is in
`docs/lessons_plan.md`'s "Wave 2" section, Unit 4. Its engine prerequisites
(jazz chord-tone tables — `ii_v_i_chords`, `ChordQuality::{Major7,
HalfDiminished7,Dominant7Alt}` — and `Progression::JazzBlues`) are done; what
remains:

- Position work beyond blues 2nd position; chromatic slide technique as a
  first-class taught skill (the `Modifier::Slide` scoring already exists —
  see `CLAUDE.md`).
- Jazz-standard chart content — same rights/judgment-sensitive gap as
  blues content (`TODO.md`), likely worse (jazz standards are more often
  still in copyright); may need to lean on public-domain jazz-blues heads
  and original content rather than standards.

## 0.7+ — "Reach" (platforms & instruments)

- A 3D harmonica prop model for chromatic charts — `Play3D` lane geometry
  already adapts to a chromatic chart's hole count, but no matching mesh
  exists yet, so the bundled diatonic model still renders.
- Packaged releases as first-class CI artifacts: Flathub submission, Windows
  installer (workflow exists), macOS bundle (workflow exists — `release.
  yaml` builds/DMGs both architectures at tag time, `macos.yaml` checks the
  same bundling on every push/PR so a regression doesn't wait for a tag).
- Explore web build (Bevy → wasm; mic via Web Audio) for zero-install
  trial. The crate compiles clean for `wasm32-unknown-unknown` (see
  `Cargo.toml`'s wasm32 target section), `trunk build`/`serve`
  (`index.html`, `Trunk.toml`) produces a real, servable bundle, and the
  app now genuinely boots and keeps running in a browser (verified with
  headless Chromium, checked for zero panics across a full run): WGPU
  initializes, localization loads (`harmonicon-platform`'s `localization.rs`'s fixed
  `LOCALES` list, loaded by explicit path instead of `AssetServer::
  load_folder`'s directory scan — the wasm HTTP asset reader can't
  enumerate a directory, which used to hard-panic `bevy_fluent` on
  startup), mic capture fails gracefully exactly like a real
  permission-less browser would (`MicStatus::Failed`, no panic), and
  bundled songs, note themes, harmonica models, and UI themes now actually
  load: `assets_management`'s directory-scanning discovery is `#[cfg(not(
  target_arch = "wasm32"))]` on native (unchanged — a player can still drop
  a new song into `assets/songs/` or `~/Harmonicon/songs/` with no rebuild)
  and reads a `build.rs`-generated manifest on wasm instead
  (`generate_wasm_asset_manifest`, `$OUT_DIR/asset_manifest.rs` — safe
  because a build script always runs on the native host regardless of
  `--target`, so it can do a real `std::fs::read_dir` walk of `assets/` even
  when building for wasm32); `theme::load_theme`'s own `theme.json` read
  went from raw `std::fs::read_to_string` (can't work over HTTP either) to
  a proper `AssetServer`-loaded `Asset` via a small custom loader
  (`theme::ThemeJsonLoader`, matching `song::loader::SongChartLoader`'s
  pattern). Verified via headless Chromium: the old "No songs directory
  found"/"No note themes directory"/"No harmonica models directory"/
  "Could not find theme.json" warnings are all gone. (The wasm pass also
  surfaced a genuine WebGL2 incompatibility in the themed buttons' smoke
  shaders — a bare `f32` uniform, which WebGL2's downlevel backend
  requires to be 16-byte aligned — resolved by removing that shader
  effect rather than patching it.) What's left: a Web Audio bridge for mic
  capture in place of `cpal`, and a replacement for the `dirs`/`std::fs`-
  based settings/profile persistence and the `~/Harmonicon` external-folder
  watcher (`notify-debouncer-full`), none of which have browser
  equivalents.
- Android: **the Rust side is done and CI-guarded; the APK is not.** See
  `docs/android.md` for the full record and `PLAN.md` for what landed. The
  workspace type-checks for `aarch64-linux-android` with no NDK, so the port
  can't silently rot, but no APK has been built, installed or run — and in
  particular nobody has confirmed the mic actually captures usably through a
  phone, which for this game is the whole product. What's left is a real
  device and a real toolchain: the APK itself, a touch/hit-target pass, icon
  and splash, and validating `[package.metadata.android]`.
  - Mic input is *not* blocked the way it is for
    wasm — checked cpal 0.17.3's own source directly, and both Android
    (`host/aaudio`) and iOS (`host/coreaudio/ios`) have real, non-stub
    input support, unlike the wasm/Web Audio backend, which has none at
    all.
  - What remains genuinely unsettled is the *packaging* toolchain, not the
    code: no single current, actively-maintained tool goes all the way to a
    store-ready build (`cargo-apk`, which the committed metadata targets, is
    explicitly deprecated and **can't produce a Play Store AAB**; the
    community's `bevy_game_template` leans on a custom fork of `xbuild`,
    itself unmaintained upstream) — a Bevy maintainer's own words: shipping
    to mobile is "possible... but not easy." Expect to revisit this choice
    the moment a Play Store release is actually wanted; none of the Rust-side
    work depends on it.
- Explore iOS. Untouched, and it needs a Mac with Xcode, unavailable in this
  dev environment. Not blocked by anything the Android work left behind: the
  asset-discovery `#[cfg]`s deliberately exclude iOS (an app bundle's
  Resources directory reads like any other, so it keeps the runtime scan and
  the drop-folder dynamism), and `permission.rs` is structured to take an iOS
  arm — `NSMicrophoneUsageDescription` needs the same
  ask-then-park-then-poll shape `RECORD_AUDIO` now has.
- Accessibility: mirrored layout for left-handed players, fully
  keyboard-navigable menus. (Colorblind-safe note palettes are done — an
  Options-page toggle swaps the Play2D/Play3D note highway's blow/draw
  colors for a fixed blue/yellow pair; themes can also set their own
  `colors.notes` block. The Song Editor, harmonica overlay legend, and
  song-progress/scrollbar minimaps still use their own hardcoded blue/
  orange, which already reads reasonably under red-green colorblindness —
  extend them to the same toggle if that turns out not to be enough.)
- Localization beyond en-US/pt-BR/es-ES (infrastructure is already enforced).

## Non-goals (for now)

- Multiplayer / online leaderboards.
- Non-harmonica instruments.
