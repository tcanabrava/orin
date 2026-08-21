# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

Harmonicon: a rhythm game for diatonic and chromatic harmonica (Rust + Bevy
0.19). The player plays a *real* harmonica into the microphone; pitches are
detected in real time and scored against a scrolling chart. Goal: teach
blues/jazz harmonica through play.

Planning docs — keep these current as work lands; prune finished items rather
than accumulating history (git log/commit messages are the historical record):
- `TODO.md` — open, actionable items only
- `ROADMAP.md` — versioned feature direction (0.4 → 0.6+)
- `PLAN.md` — execution order and implementation notes for what's in flight
- `docs/lessons_plan.md` — curriculum design for the Lessons feature
- `docs/gameplay_validation.md` — manual + automated validation checklist;
  update it when changing gameplay/timing behaviour
- `docs/book/` — player-facing mdBook user guide (`mdbook build`/`mdbook
  serve` from that directory); update it when a user-visible feature
  changes, not just internal ones. `docs/book/src/images/*.png` are
  placeholder screenshots (script-generated captioned frames) pending real
  captures — keep the filenames stable when swapping them in so the
  `![...](images/foo.png)` references throughout `docs/book/src/*.md`
  don't need touching.

## Commands

```bash
cargo run --features dev        # local iteration (dynamic linking + asset watcher)
cargo run --release             # playable build; never ship the dev feature
cargo test --workspace          # ~1100 tests; safe headless
cargo clippy --workspace --all-targets -- -D warnings   # what CI runs

# Working on pure logic? Skip the engine entirely — seconds, not a minute:
cargo test -p harmonicon-core   # ~200 tests, no Bevy in its dependency tree
# Profiling: start the Tracy UI (https://github.com/wolfpld/tracy), click
# "Connect", then:
cargo run --release --features trace_tracy
```

Binaries: main game (`src/main.rs`), plus `hole-editor`, `note_editor`,
`note_bench`, `gen_synthetic_dataset` (in `src/bin/`). The root package is
*only* the binary — every library lives in `crates/`.
Manual testing needs a mic, audio out, and a display.

## Architecture (load-bearing facts)
- **Cargo workspace — eleven library crates plus a binary-only root
  package.** A crate may depend only on ones *earlier* in this list, and
  **peers may not depend on each other**:

  | Crate | Holds | Bevy? |
  |---|---|---|
  | `harmonicon-core` | music theory, chart types, scoring math, pitch/MIDI conversion, the harmonica synth, WAV, grid snapping | **no** |
  | `harmonicon-audio` | cpal capture, FFT pitch detection, waveform analysis | yes |
  | `harmonicon-platform` | asset discovery, localization, settings, theme, responsive | yes |
  | `harmonicon-song` | chart/manifest loading, MIDI-backed songs, lessons | yes |
  | `harmonicon-app` | state machine, routing flags, profile | yes |
  | `harmonicon-ui` | `dialogs`, `music_score`, `spectrogram` | yes |
  | `harmonicon-gameplay` | clock, judging, 2D/3D highways, overlays, bend trainer | yes |
  | `harmonicon-jam` / `harmonicon-editor` | Jam Session / Song Editor — **siblings**, neither imports the other | yes |
  | `harmonicon-menu` | page state machine, routing, one file per screen | yes |
  | `harmonicon-bench` | pitch-detection benchmark + dataset generator (dev tooling) | yes |
  | `harmonicon` (root) | `main.rs` + `src/bin/*`; owns `assets/`, `build.rs`, `tests/` | yes |

  - **Keep `harmonicon-core` Bevy-free.** Its whole dependency tree is
    `serde`/`serde_json`/`midly`, which is why its ~200 tests run in
    seconds. Anything needing `Resource`/`Component`/`App` belongs a level
    up. This is the single most valuable property of the split — don't
    trade it away for convenience.
  - **No re-export facades.** A call site names the crate it depends on
    (`harmonicon_core::chart`, `harmonicon_gameplay::gameplay::…`), so every
    dependency is visible where it's taken. Re-exporting a moved module
    under its old path was tried and deliberately removed: it hid which
    crate code came from and let modules reach for things casually.
  - **Cross-crate ordering goes through a `SystemSet`, never a system
    name.** `.after(some_private_fn)` forces the owning crate to make the
    system *and its parameter types* public. `dialogs::combobox::
    ComboboxEscapeSet` and `gameplay::plugin::MusicVolumeSet` exist for
    exactly this — publish an ordering point, keep the implementation
    private.
  - **A crate cycle is not expressible**, so Cargo enforces the layering.
    Rust still permits cyclic *modules* inside one crate, which is what
    `tests/physical_design.rs::no_module_dependency_cycles` catches (no
    allowlist — see `docs/physical_design_plan.md`).
  - **Paths reaching `assets/` from a crate need `../../`.** `include_str!`/
    `include_bytes!` resolve relative to the source file, and
    `env!("CARGO_MANIFEST_DIR")` now points at the crate, not the repo.
    A wrong `include_*!` is a compile error; a wrong runtime path is not,
    so tests that read `assets/` must build the path explicitly rather than
    relying on the working directory.
  - **`assets_management`'s wasm manifest is generated by
    `harmonicon-platform`'s own `build.rs`**, because
    `include!(concat!(env!("OUT_DIR"), …))` reads the *including* crate's
    `OUT_DIR` and `OUT_DIR` is per-package. The workspace-root `build.rs`
    keeps the source-scanning lints (localization, message registration),
    which walk `src/` **and** every `crates/*/src/`.
  - **A new crate must forward the `dev`/`trace_tracy` features** to its own
    `bevy` (`"harmonicon-x/dev"`), or feature unification breaks and the
    tree ends up with two differently-configured Bevy builds.
- **Profiling/tracing is Tracy-based** (`cargo run --release --features
  trace_tracy`, per the Commands section above). `trace_tracy` (`Cargo.toml`)
  just forwards to `bevy/trace_tracy`, which already wraps every ECS system
  call in its own `info_span!("system", name = ..)` — most of what shows up
  in Tracy needs no manual instrumentation at all. Two things this crate adds
  on top:
  - `main.rs`'s `LogPlugin` is feature-gated: the everyday filter
    (`"warn,bevy_render::camera=error"`) sets the *default* level below
    `info`, which silently drops every span (Bevy's own and ours) before any
    backend — Tracy included — ever sees them (see
    `docs/profiling.md`/`LogPlugin::build_filter_layer`, which folds
    `filter`'s own bare directives over `level`). A `trace_tracy` build swaps
    in a filter with no bare-level directive below `info`, so the configured
    `Level::INFO` default actually holds.
  - Manual spans cover the paths automatic per-system instrumentation can't
    reach — anything that isn't itself a system call. Two categories so far:
    - **Off the ECS schedule entirely:** the cpal capture callback
      (`audio_input::push_chunks`) runs on its own real-time thread; the only
      custom `AssetLoader` (`song::loader::SongChartLoader::load`) runs as a
      future on the AssetServer's IO task pool. Both get a manual span for
      the same reason — Bevy's per-system spans only wrap systems the
      schedule itself calls, so anything running elsewhere (another thread,
      another executor) is otherwise invisible no matter how expensive it
      is. A span held across an `.await` needs `tracing::Instrument` (via
      `bevy::log::tracing::Instrument`) rather than a plain `.entered()`
      guard — an `EnteredSpan` isn't `Send`, which the loader's returned
      future must be; `SongChartLoader::load` is a thin wrapper that
      instruments a `load_inner` for exactly this reason.
    - **A hot inner loop worth breaking out of its system's own total time:**
      `pipeline::process_audio`'s per-chunk work; `pitch_detect::analyze`'s
      FFT transform and per-algorithm dispatch; `build_nmf_dict` (the
      priciest one-off, rebuilt only when the NMF dictionary goes stale);
      `waveform::analyze_ogg_waveform`/`analyze_wav_waveform` (a whole-file
      decode — also called from the off-schedule asset loader above, so it
      carries both reasons at once).
    Add spans the same way for any other code that runs off the main
    schedule (more asset loaders, decode threads, the asset watcher — though
    `assets_management::watch`'s debouncer thread runs only
    `notify-debouncer-full`'s own code, nothing of ours, so there's nothing
    to instrument there) or burns real time inside a single system call.
- **Per-crate architecture notes live in `crates/<name>/CLAUDE.md`.** Each
  crate documents its own load-bearing facts; they load when you're working
  in that crate rather than all at once. Start there for anything
  subsystem-specific:

  | Crate | Its `CLAUDE.md` covers |
  |---|---|
  | `harmonicon-core` | (no separate file — pure logic, documented at its `//!` headers) |
  | `harmonicon-audio` | the cpal→FFT input path, chart-driven detection range |
  | `harmonicon-platform` | asset sources + the `~/Harmonicon` watcher, settings, compact layout |
  | `harmonicon-song` | chart format, schema migration, optional sibling assets, lessons |
  | `harmonicon-app` | `AppState`/`MenuPage` states, profile persistence |
  | `harmonicon-ui` | the Bravura notation staff |
  | `harmonicon-gameplay` | clock/time authority, scoring, adaptive difficulty, HUD, progress bar, call-and-response |
  | `harmonicon-jam` | generated manifests, jam lesson criteria, freeform call-response, MIDI stems |
  | `harmonicon-editor` | the whole Song Editor (record, MIDI import, undo, timeline tools, tempo map) |
  | `harmonicon-menu` | the guided tutorial tour, menu page scrolling |
  | `harmonicon-bench` | benchmark-first pitch-detection workflow |

## Conventions (enforced or established)

- **UI is authored with `bsn!`** wherever applicable; widget callbacks go
  inline as `on(...)` observers — not `Changed<Interaction>` systems, not
  imperative `spawn().observe()`. Prefer `bevy_ui_widgets` over hand-rolled
  widgets; buttons go through the shared `dialogs/button.rs` widget unless
  there's a real reason not to. For a destructive action needing "are you
  sure?", use `dialogs::confirm_dialog` (`OpenConfirmDialog{purpose,
  message}` in, `ConfirmChosen{purpose, confirmed}` out — same
  message-based, `DialogId`-scoped shape as `dialogs::file_dialog`) rather
  than firing immediately or hand-rolling another modal; the Song Editor's
  Erase/Remove timeline tool (`song_editor::timeline`) is its first user.
- **Keyboard navigation:** every interactive element must use a real
  `bevy_ui_widgets` widget (explicit `use bevy::ui_widgets::Button as
  WidgetButton` — plain `bevy::prelude::*` resolves the bare `Button` name
  to `bevy_ui`'s *legacy*, pre-headless-widgets marker instead, which has
  no keyboard support at all) with `TabIndex(0)` attached, never a
  hand-rolled `Pointer<Click>` observer on a plain `Node`. Every screen's
  root needs a `TabGroup` (`bevy::input_focus::tab_navigation`) for
  Tab/Shift+Tab to scope to; a modal (a confirm/file dialog, an open
  combobox dropdown) needs `TabGroup::modal()` or its items' `TabIndex`
  flipped negative while closed (see `dialogs::combobox::
  set_combobox_open`) so Tab can't reach something invisible — bevy's own
  tab-gathering walks the ECS tree by `TabIndex`/`Children` alone, with no
  `Display`/`Visibility` check. `dialogs::keyboard_nav::KeyboardNavPlugin`
  registers `TabNavigationPlugin` (not in `DefaultPlugins`, unlike
  `InputFocusPlugin`) and paints the focus ring; it does **not** bridge
  `Activate` to `Pointer<Click>` — every click handler on a real
  `WidgetButton` is written directly as `On<Activate>` (`Activate` is what
  `bevy_ui_widgets::Button` fires for both a real click and a focused
  Enter/Space, so one handler covers both). An earlier version routed
  `Activate` through a synthetic re-triggered `Pointer<Click>` so ~130
  existing handlers wouldn't need retyping — that bridge could recurse
  into `bevy_ui_widgets`' own `button_on_pointer_click` (which reacts to
  the synthetic click and, seeing the real click's still-`Pressed`
  component, re-emits `Activate`) and overflow the stack on an ordinary
  mouse click. A new button-shaped click handler must therefore take
  `On<Activate>`, not `On<Pointer<Click>>` — and only ever on an entity
  that actually carries a real `WidgetButton`; retyping a handler on a
  legacy `bevy_ui::widget::Button` (or on a plain `Node` with no Button at
  all, e.g. the Song Editor's note-grid drag surfaces or harmonica-diagram
  cells) to `On<Activate>` doesn't fail to compile — it just silently
  never fires, since neither ever emits `Activate`. Multi-option pickers
  (radio-shaped, not "click to advance") should use `RadioGroup`/
  `RadioButton` (see `dialogs::tab_bar`) rather than a hand-rolled
  mutually-exclusive button row — only the group itself gets `TabIndex`,
  per WAI-ARIA convention (individual radio buttons aren't Tab stops; the
  group's own arrow-key handling reaches them). `Checkbox` needs no
  `Button` alongside it — it already has independent Enter/Space and
  click handling that doesn't go through `Activate`, and adding `Button`
  risks a double-toggle (see `dialogs::checkbox.rs`).
- **Bevy 0.19 scene spawning:** use `WorldAssetRoot(handle)` for GLB/scene
  assets, not `SceneRoot`.
- **Localization is enforced:** user-visible strings must come from
  `loc.msg()` (Fluent); a `build.rs` scan + `LocalizedStr` newtype fail the
  build on raw literals. Locales: en-US, pt-BR, es-ES — add keys to all;
  `locales_define_the_same_keys` walks the directory and enforces parity.
  **A key with a variable uses Fluent's own `{$name}` syntax** (e.g.
  `jam-generate-key = Key: {$key}`) — `LocalizationExt::msg_args`
  (`localization.rs`) builds a real `fluent::FluentArgs` from the
  `&[(&str, String)]` it's given and resolves it through
  `fluent_content::Request::args`/`Content::content`, i.e. Fluent's own
  `format_pattern`, not a hand-rolled string replace. One wrinkle:
  Fluent wraps every interpolated argument in bidi-isolation marks
  (FSI/PDI, U+2068/U+2069) by default, meant for prose mixing scripts —
  overkill for short single-language UI labels, and `bevy_fluent`'s
  bundle loader exposes no setting to turn it off
  (`FluentBundle::set_use_isolating` isn't reachable through it), so
  `msg_args` strips those marks from the result via the pure, tested
  `strip_bidi_isolates` helper rather than let invisible formatting
  characters leak into rendered/logged text. `msg` (no args) skips all of
  this — it's a plain `self.content(key)` lookup, no `FluentArgs` involved.
  Runtime *loading* of those locales (`localization::load_locales`) is a
  fixed `LOCALES` list, each loaded by explicit path
  (`locales/<lang>/main.ftl.ron`), not `AssetServer::load_folder` — the
  wasm build's HTTP asset reader can't enumerate a directory
  (`bevy_asset::io::wasm::HttpWasmAssetReader`), and `load_folder` needing
  exactly that is what used to hard-panic the game on startup under wasm
  (`bevy_fluent`'s `LocalizationBuilder::build` indexing an empty map).
  `locales_const_matches_the_assets_directory` keeps `LOCALES` honest
  against what's actually on disk, since nothing else does anymore now
  that nothing scans the directory at runtime. The same directory-listing
  constraint applies to anything else that might run under wasm.
  `assets_management`'s song/note-theme/harmonica-model discovery
  (`scan_all_songs`, `scan_note_themes`, `scan_harmonica_models`,
  `scan_ui_themes`) takes the build-time-manifest approach instead: each is
  now two `#[cfg]`-gated implementations under the same name — the original
  `std::fs::read_dir`-based body, unchanged, behind
  `#[cfg(not(target_arch = "wasm32"))]` (this is what keeps native dynamic:
  a player can still drop a new song into `assets/songs/` or
  `~/Harmonicon/songs/` with no rebuild — a fixed manifest like
  localization's `LOCALES` isn't an option here, unlike the fixed set of
  three shipped locales), and a `#[cfg(target_arch = "wasm32")]` sibling
  that reads a `build.rs`-generated manifest instead
  (`generate_wasm_asset_manifest`, `assets_management::manifest`'s
  `include!(concat!(env!("OUT_DIR"), "/asset_manifest.rs"))`). Generating
  that manifest at build time — rather than runtime — works specifically
  because a build script always runs on the native host and can do a real
  `std::fs::read_dir` walk of `assets/` regardless of the crate's own
  `--target`; `build.rs`'s `generate_wasm_asset_manifest` mirrors each scan
  function's discovery rule exactly (e.g. the first `*.harpchart` file
  directly under a song's `song/` subfolder) so the two implementations
  can't drift. The `~/Harmonicon` external-folder equivalent has no wasm
  version at all — no home directory concept in a browser — which the
  native functions already handle gracefully (`dirs::home_dir()` returning
  `None`), so nothing wasm-specific was needed there. UI *theme* content
  (not just names) also loads under wasm now: `theme::load_theme` used to
  read `theme.json`'s contents via a raw `std::fs::read_to_string`, which
  can't work under wasm either (a different mechanism than a directory
  listing — an actual file read) — `ThemeJson` is now an `Asset` loaded
  through a small custom `ThemeJsonLoader` (`AssetLoader`, matching
  `song::loader::SongChartLoader`'s pattern; registered by full filename,
  `extensions() -> &["theme.json"]`, so it can't collide with some other
  `.json` asset gaining its own loader later), fetched the same way
  `load_theme` already loaded its sibling images/sounds. Loading is now
  two systems instead of one synchronous function: `request_theme_load`
  (on `SelectedTheme` change) clears `LoadedTheme` and kicks off the
  `asset_server.load::<ThemeJson>(...)` call, stashing the `Handle` in a
  `PendingTheme` resource; `apply_theme_when_loaded` polls it every frame
  (a no-op whenever `PendingTheme` is absent) and populates `LoadedTheme`
  once the load resolves. `theme_source_prefix` (bundled vs.
  `external://`) is also `#[cfg]`-split the same way as the
  `assets_management` scan functions above — its native body does a real
  `Path::is_dir()` check, which needs a real local filesystem wasm doesn't
  have; the wasm sibling always answers "bundled", correct because
  `AvailableThemes` under wasm only ever lists bundled themes to begin
  with (same no-external-folder reasoning as everywhere else here).
- **Message registration is enforced:** `build.rs` also scans for every
  `#[derive(Message)]` type and fails the build if it's never registered
  with `.add_message::<T>()` anywhere — an unregistered message otherwise
  compiles fine and only panics at runtime ("Message not initialized") the
  first time some system's `MessageReader`/`MessageWriter` for it actually
  runs, which can be well after the type was added.
- **Audio synthesis:** vibrato/FM must integrate frequency over time (phase
  accumulation), never `modulated_freq × t` — the latter drifts pitch upward.
- **Testing style:** new mechanics get pure functions + unit tests first,
  ECS systems second. Scoring/chart/pitch logic all have dense test modules —
  match that. ECS behaviour is tested with minimal `World` + `Schedule` or
  `App` + `StatesPlugin` (see `menu/mod.rs`, `gameplay/tests.rs`).
- **Commits:** no `Co-Authored-By` trailer. Chart schema changes must stay
  backward compatible (new fields optional); bump `metadata.format_version`.

## Known open items

- Content: besides the Example Artist gameplay demos, bundled songs now
  include public-domain melodies (Greensleeves on a G harp, Jesu Joy and
  the Toccata in D minor on C harps, Für Elise on a C chromatic,
  "O Pulo da Gaita" transcribed from the Mr. Dirsom harmonica tab score,
  Amazing Grace, the Hallelujah chorus from Handel's Messiah on a D harp,
  and Mulher Rendeira). `tests/asset_layout.rs` schema-validates every
  bundled song chart. Deliberately skipped as still under copyright:
  Feira de Mangaio (Sivuca/Glorinha Gadelha) and Asa Branca (Luiz
  Gonzaga/Humberto Teixeira) — chart those yourself via Record mode
  instead of bundling a transcription.
- **Song editor color legend**: a third meta-form column
  (`meta_form::spawn_color_legend`) explains every color the editor uses,
  grouped by where it appears — note technique colors in the grid
  (`state::pitch_color`; direction is the ↑/↓ arrow glyph, not a color),
  the out-of-scale red tint, the selected-note border, drag-ghost valid/
  invalid, and the timeline/scrollbar colors — deliberately calling out
  that the scrollbar minimap's blue/orange means blow/draw
  (`interaction::SCROLLBAR_BLOW_COLOR`/`SCROLLBAR_DRAW_COLOR`), a
  different meaning than the grid note's blue (which means the Normal
  technique, regardless of blow/draw). Several colors that were private
  `const`s or local `let` bindings (`grid::OUT_OF_SCALE_TINT`/
  `TEMPO_MARKER_COLOR`, `timeline_overlay::SPLIT_LINE_COLOR`/
  `RANGE_HIGHLIGHT_COLOR`) were widened to `pub(super)` so the legend
  reuses the exact values instead of duplicating literals that could
  drift out of sync.
- **Song editor: selectable scale** (`song::chart::Scale`, a new chart
  field): the grid's out-of-scale red tint used to always mean "outside
  the blues scale rooted on the harp key" unconditionally
  (`blues_scale_classes(&state.key)`); it's now `state.scale.classes(&state.
  key)`, `state.scale` picked via a combobox (`meta_form::
  spawn_scale_combobox`) — six options: 1st/2nd/3rd position (the blues
  hexatonic, same shape as everywhere else, just rooted at the harp key
  \+0/+7/+2 semitones — the same offsets `Position::interval_below_jam_key`
  uses for Jam Session's harp-picking, just applied upward from the harp's
  own key instead of downward from a separate jam key, since a chart has
  no jam key distinct from its harp) and Major/Minor Pentatonic/Country
  (alternative *shapes*, always rooted on the harp key — for melodies that
  aren't blues-vocabulary at all; "Country" = major pentatonic, the
  scale 2nd-position cross-harp playing reaches without bending, per
  harmonica-pedagogy convention). `FirstPosition` (the default, used when
  a chart doesn't set `scale` at all) reproduces the old unconditional-
  blues behavior exactly — `first_position_matches_blues_scale_classes_
  exactly` pins this down. `harmonica.scale` is a new, schema-`enum`-
  validated field (unlike its free-string `position` sibling), added to
  both `Harmonica::Diatonic`/`::Chromatic`; `CURRENT_FORMAT_VERSION`
  bumped to 1.2.0 since an older build's stricter schema would otherwise
  reject a chart that actually sets it with a confusing raw validation
  error instead of the intended "needs a newer Harmonicon" message — a
  chart that never sets `scale` needs no version bump, unaffected either
  way. The combobox itself is spawned once into a reserved
  `ScaleComboboxSlot` (`spawn_scale_combobox`, a `Without<Children>`
  spawn-once gate, unlike the MIDI track combobox's rebuild-on-message
  pattern, since `Scale::all()`'s option list never changes at runtime);
  Load pushes a different value into the already-spawned combobox by
  writing `ComboboxValue` directly (`sync_scale_combobox_value`) — the
  widget's own documented escape hatch for exactly this, `dialogs::
  combobox`'s always-on `sync_combobox_visuals` picks the change up from
  there. No existing bundled chart sets `harmonica.scale` — all keep
  reading as 1st position, i.e. unchanged from before this feature.
  **`ScaleComboboxSlot` lives in the fixed chrome** (`ui::
  spawn_fixed_chrome`, above the mod panel — not the scrollable meta form
  the rest of the fields are in), a deliberate, load-bearing placement:
  `bevy_ui_widgets::Popover`'s dropdown list must be a literal ECS child of
  its toggle to compute its own position, and Bevy's UI overflow clipping
  follows that same ancestry rather than the popover's computed screen
  position — a combobox nested inside the form's `Overflow::scroll_y()`
  `ScrollArea` gets its open dropdown clipped to that scroll viewport no
  matter how high its `GlobalZIndex` is, rendering behind (and stealing
  clicks from) whatever's in the unclipped fixed chrome instead. The MIDI
  track combobox has this same latent constraint (it's also inside that
  `ScrollArea`) but hasn't surfaced as a visible bug yet — if it ever does,
  the fix is the same: move its slot out of the scrollable area too.
  **Fixing the clipping surfaced a second, separate bug in `dialogs::
  combobox` itself, affecting every combobox, not just Scale's**:
  `Pointer<Click>` auto-propagates up the entity hierarchy (every
  `bevy_picking` pointer event does, `#[entity_event(propagate =
  PointerTraversal, auto_propagate)]`) — clicking a dropdown item bubbled
  the same click up to the toggle button (`list`'s ancestor), whose own
  `toggle_click` observer then saw the popup `item_click` had *just* closed
  and immediately reopened it, so picking an item never visually closed
  the dropdown. Fixed by calling `ev.propagate(false)` in all three of the
  widget's own click observers (`toggle_click`/`backdrop_click`/
  `item_click`) — a modal widget shouldn't leak its own clicks to whatever
  it happens to be nested inside, regardless of this specific bug.
- **Lessons**: engine, all five primitives, and the full wave 1 + wave 2
  content pass (Units 1–3, 19 lessons) are shipped — see
  `docs/lessons_plan.md`. Unit 4 "jazz"'s engine prerequisites are also done
  (`song::harmonica::ii_v_i_chords`, `ChordQuality::{Major7,
  HalfDiminished7,Dominant7Alt}`, `Progression::JazzBlues`); what's left is
  content only, the same rights/judgment-sensitive gap as blues content
  (`TODO.md`).
- Remaining 0.4 work (recorded backing loops) — see `ROADMAP.md`/`PLAN.md`.
