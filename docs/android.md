# Android

## Status: the Rust side compiles; the APK has never been built

Be precise about which half of this is known to work.

**Verified.** `cargo check --target aarch64-linux-android` type-checks the
entire workspace, and CI's `android_check` job keeps it that way. That covers
every code change listed below: the entry point, asset discovery, the
permission flow, and the desktop-only paths that had to be gated.

**Not verified.** No APK has been built, installed, or run. Producing one
needs the Android NDK, SDK and a JDK, none of which were available where this
was written. So everything in `[package.metadata.android]`
(`crates/harmonicon-android/Cargo.toml`) — SDK levels, orientation, the
manifest permission, icon handling — is a reasoned starting point, not
settled configuration. Expect to adjust it on first contact with a device.

In particular, **nobody has confirmed the microphone actually works on
Android**, which for this game is the whole product. cpal has real Android
input support and the permission request is wired up, but "compiles" and
"captures a harmonica through a phone mic with usable latency" are very
different claims.

## Building an APK

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-apk
export ANDROID_HOME=/path/to/android/sdk
export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/<version>
cargo apk run -p harmonicon-android
```

`cargo-apk` is unmaintained but is what matches the `native-activity`
backend chosen below. If it proves to be a dead end, the alternatives are
`xbuild` or `cargo-ndk` driving a hand-written Gradle project — both of which
would replace `[package.metadata.android]` with their own configuration and
leave every Rust-side change here untouched.

## Why the code is shaped the way it is

### `crates/harmonicon-android` — a separate cdylib crate

Android does not call `main`. The platform loads a shared library and calls
`android_main`, so the app needs a `cdylib` target. That forced the
composition root out of `src/main.rs` and into `src/lib.rs`'s `run()`, which
both entry points now call — the root package is no longer binary-only.

The cdylib lives in its own crate rather than as a second `crate-type` on the
root package because `default-members` includes every workspace member: a
cdylib on the root would relink the entire Bevy app on every desktop
`cargo build`. Instead, `harmonicon-android`'s dependency on the game is
target-gated and its `src/lib.rs` is entirely `#[cfg(target_os = "android")]`,
so off Android it compiles to an empty cdylib with no dependencies.

### `native-activity`, not `game-activity`

`android-activity` requires exactly one backend, and Bevy 0.19's default
features select neither (its defaults are just `2d`/`3d`/`ui`/`audio`), so
this had to be chosen explicitly.

`native-activity` was picked partly on merit — it is pure Rust, and it is what
`cargo-apk` supports — and partly by constraint: `game-activity` compiles
`GameActivity.cpp` from the NDK, which cannot be built here, so choosing it
would have meant the Rust side could not be checked at all.

**This is the decision most likely to need revisiting.** GameActivity has
materially better soft-keyboard and IME handling, which matters for the Song
Editor's text fields. If text entry on a phone turns out to be unusable,
switching backends is a one-line feature change plus whichever packaging tool
supports it.

### blake3's pure-Rust backend

`blake3` (via `bevy_animation`) compiles a C SIMD backend by default, needing
`aarch64-linux-android-clang` from the NDK. Its `pure` feature is selected for
`cfg(target_os = "android")` only, so desktop keeps the fast path.

This is what makes an NDK-free `cargo check` — and therefore CI's
`android_check` job — possible at all. It also means the Android build hashes
assets more slowly than desktop; if that ever shows up in a profile, an
NDK-equipped build machine can simply drop the feature.

### Asset discovery goes through the build-time manifest

An APK's assets live inside the archive, reachable only through the JNI
`AssetManager` — `std::fs::read_dir("assets/songs")` returns `Err`, and the
runtime scans would silently find nothing at all.

This is the same constraint wasm already had, so Android reuses the same
solution: the `#[cfg]`-split scan functions in
`harmonicon-platform`'s `assets_management`, backed by a `build.rs`-generated
manifest. The cfgs are now
`any(target_arch = "wasm32", target_os = "android")`.

Two consequences worth knowing:

- **Lessons were broken on wasm before this**, not just on Android.
  `lessons::catalog` had no manifest path at all, because it reads each
  `lesson.json`'s bytes directly rather than through `AssetServer`. It now
  has one (`crates/harmonicon-song/build.rs`), which embeds the manifest text
  with `include_str!` rather than just directory names. Fixing Android fixed
  wasm.
- **iOS is deliberately not included.** An app bundle's Resources directory
  reads like any other directory, so iOS keeps the runtime scan and gets the
  `~/Harmonicon`-style dynamism for free.

### The microphone permission

`RECORD_AUDIO` is a "dangerous" permission: declaring it in the manifest only
makes it requestable, and until the user grants it at runtime, opening a cpal
input stream fails in a way indistinguishable from a broken device.

`harmonicon-audio`'s `permission` module calls
`Activity.checkSelfPermission`/`requestPermissions` over JNI.
`audio_input::start_capture` asks first and parks in
`MicStatus::AwaitingPermission` — a state that already existed as groundwork,
and which the Options page already renders a banner for — and
`retry_capture_when_permission_granted` polls until the dialog is answered.

It polls because the result is delivered to an `onRequestPermissionsResult`
callback on a Java activity this codebase doesn't own. A once-per-install
dialog does not justify routing that back across JNI.

Off Android, `microphone_granted()` returns `true` unconditionally and
`request_microphone()` does nothing, so the call sites need no `#[cfg]`.

## What has *not* been done

- **No APK build, install, or device run** — the whole "Not verified" section
  above.
- **No touch-input pass.** Keyboard-only actions already have on-screen
  equivalents (that groundwork landed earlier), but nothing has been sized or
  laid out for a thumb. Hit targets, the Song Editor's dense timeline, and
  drag surfaces all want a real device to judge.
- **No icon or splash screen.**
- **Opening the user guide does nothing on Android.**
  `help_about::open_in_default_app` returns `Unsupported`, which the UI
  reports honestly. Doing it properly means handing an `Intent` to the system
  over JNI.
- **iOS remains untouched.** Nothing here blocks it, and the asset-discovery
  work explicitly leaves it on the native path, but it needs Xcode.
