# Driving a running game from outside

`--features dev` starts a [Bevy Remote Protocol][brp] server — JSON-RPC over
HTTP on `127.0.0.1:15702` — so a running game can be inspected, mutated,
screenshotted and recorded from a shell, with no rebuild and no code added
at each interesting moment. Wiring is `src/dev_capture.rs`.

**Never shipped.** BRP is unauthenticated and can read and mutate arbitrary
world state. `dev` is a compile-time feature, so a release build doesn't
merely disable it, it doesn't contain it.

[brp]: https://docs.rs/bevy_remote

## Running

```bash
cargo run --features dev
```

Launching the built binary *directly* needs one extra thing, or every asset
fails to load and you get a blank window: Bevy resolves `assets/` relative to
the **executable** (`target/debug/assets`), not the working directory, unless
`CARGO_MANIFEST_DIR` is set — which `cargo run` does and a bare `./harmonicon`
does not.

```bash
BEVY_ASSET_ROOT="$PWD" ./target/debug/harmonicon
```

(`configured_asset_plugin` in `src/lib.rs` special-cases this for macOS debug
builds only; everywhere else, use one of the two forms above.)

A helper for the examples below:

```bash
brp() { curl -s -X POST http://127.0.0.1:15702 \
        -H 'Content-Type: application/json' -d "$1"; }
```

## Screenshots → `target/screenshots/`

Bevy's own `Screenshot` component is `Reflect` and registered, so BRP can
spawn one. `dev_capture`'s global observer writes whatever gets captured:

```bash
brp '{"jsonrpc":"2.0","id":1,"method":"world.spawn_entity","params":
     {"components":{"bevy_render::view::window::screenshot::Screenshot":
     {"Window":"Primary"}}}}'
```

Files are named `shot_<unix_millis>.png`, so repeated captures accumulate
rather than overwriting. Deliberately not bevy's `save_to_disk`, which takes
one fixed path.

## Video → `target/video/`

Set `frames_left`; one frame is captured per rendered frame until it reaches
zero. Each recording gets its own numbered directory.

```bash
brp '{"jsonrpc":"2.0","id":2,"method":"world.mutate_resources","params":
     {"resource":"harmonicon::dev_capture::VideoCapture",
      "path":".frames_left","value":300}}'
```

The game writes numbered PNGs; encoding is left outside, since pulling a
video encoder into the dependency tree for a dev tool isn't worth it:

```bash
ffmpeg -y -framerate 30 -i target/video/0001/frame_%06d.png \
       -c:v libx264 -pix_fmt yuv420p target/video/0001.mp4
```

**Capturing every frame stalls the render loop** — each frame is a GPU
readback. Expect the recorded clip to run slower than real time, and don't
use it to judge performance or timing. It shows *what* happened, not *how
fast*.

## Inspecting and mutating state

```bash
# What the menu actually says right now
brp '{"jsonrpc":"2.0","id":3,"method":"world.query","params":
     {"data":{"components":["bevy_ui::widget::text::Text"]},"filter":{}}}'

# Everything reachable
brp '{"jsonrpc":"2.0","id":4,"method":"world.list_components"}'
brp '{"jsonrpc":"2.0","id":5,"method":"world.list_resources"}'
brp '{"jsonrpc":"2.0","id":6,"method":"rpc.discover"}'
```

Type paths are the full Rust paths and they matter: UI text is
`bevy_ui::widget::text::Text`, **not** `bevy_text::text::Text` (which exists,
is registered, and matches nothing on a UI node).

`world.mutate_components`/`world.mutate_resources` change state live, and
`world.write_message`/`world.trigger_event` fire messages and events without
synthesising input — useful precisely where synthetic input is unreliable
(see `docs/android.md` on sub-frame taps being dropped).

### The catch: only reflected, registered types are visible

BRP reaches everything through `AppTypeRegistry`. Bevy's own components are
registered, which covers the whole UI tree, text, transforms and windows.
**This codebase's own types are not** — `EditorToolbar`, `MicStatus`,
`Scroll`, `AppState` and friends are plain `#[derive(Component)]`/`Resource`,
so `world.list_resources` shows exactly one Harmonicon type today:
`VideoCapture`, which derives `Reflect` and calls `register_type` precisely
so it can be driven from outside.

Add `#[derive(Reflect)]` + `app.register_type::<T>()` per type as the need
arises, rather than blanket-deriving it.

## What this does *not* give you

- **Rendering correctness.** BRP reports the *string* `"♬ Import MIDI"`
  whether or not the font has the glyph. Five tofu boxes shipped in every
  locale for months and only a screenshot caught them. Use both.
- **Timing.** See the readback stall above.
- **Audio.**

## Android

The same server runs in the Android build (`--features dev`), reachable by
forwarding the port:

```bash
adb forward tcp:15702 tcp:15702
```

The capture directories are then inside the app's sandbox rather than
`target/`, so pull them with `adb`.
