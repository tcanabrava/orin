# Android

## Status: the APK builds; it has not been run on a device

**Verified.** `packaging/android` produces a real, installable, signed APK
(147 MB), and its contents have been inspected rather than assumed: the
arm64 cdylib is present and exports both `android_main` and
`GameActivity_onCreate`, `com.google.androidgamesdk.GameActivity` is in
`classes.dex`, `RECORD_AUDIO` is declared, all 186 asset entries are packaged
(103 lesson files, 21 song files) and the dev-only `debug_songs` corpus is
excluded. CI's `android_check` job type-checks the target on every push.

**Not verified.** Nobody has installed or launched it. In particular **nobody
has confirmed a phone microphone actually captures usably**, which for this
game is the whole product. Everything below the build is still unknown:
touch targets, whether the landscape-only lock is right, real frame rates,
and whether the Song Editor is usable at all on a phone.

## Building

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
cd packaging/android
./gradlew assembleRelease          # -> app/build/outputs/apk/release/app-release.apk
./gradlew installRelease           # with a device connected via adb
```

Requires the SDK (platform 35, build-tools 35.0.1), **NDK 28.2.13676358**, a
JDK 17+ (Android Studio's bundled `jbr` works), and `cargo-ndk`. Gradle comes
from the committed wrapper. If your SDK isn't at `$ANDROID_HOME`, put
`sdk.dir=/path/to/sdk` in `packaging/android/local.properties` (gitignored).

The Gradle build invokes `cargo ndk` itself — there is no separate Rust step
to remember. Expect ~6 minutes for a cold Rust release build.

## Decisions worth knowing

### GameActivity, and why its version is not free to choose

`android-activity` requires exactly one backend and Bevy 0.19 selects
neither by default (its defaults are just `2d`/`3d`/`ui`/`audio`), so this
had to be chosen explicitly. GameActivity was picked over NativeActivity for
its far better soft-keyboard/IME handling, which the Song Editor's text
fields need.

**The Java AAR version must match the C++ vendored in the Rust crate.**
`android-activity`'s `GameActivity.h` declares
`GAMEACTIVITY_MAJOR/MINOR/BUGFIX_VERSION` as `4`/`4`/`0`, so the Gradle
dependency is pinned to `androidx.games:games-activity:4.4.0`. A mismatch
fails at *runtime* — `RegisterNatives` aborts the process — not at build
time. Re-read those defines before bumping `android-activity`.

### API 28 is a hard floor, not a preference

cpal's Android backend links `libaaudio`, which only exists in the NDK
sysroot from API 26 up. Below that the link fails with a bare `unable to
find library -laaudio` that says nothing about why. `minSdk` in
`app/build.gradle.kts`, the `-P` passed to cargo-ndk, and CI's check all
have to agree; 28 also comfortably clears the API 23 floor for the runtime
permission API `permission.rs` calls.

### The Android-only Cargo config lives in `harmonicon-android`

It used to sit on the root package, which meant `cargo ndk -p
harmonicon-android` silently dropped it and produced a build with no
activity backend. It now lives in the crate that *is* the Android target, so
`-p` works. Keep it there.

Note `cargo-ndk` spells platform `-P`; `-p` is cargo's package flag. Passing
`-p 28` gets you `unknown package: 28`.

### No `[package.metadata.android]`

That block belongs to `cargo-apk`, which is deprecated and cannot emit a Play
Store AAB. Gradle can (`./gradlew bundleRelease`), which is why the packaging
went this way.

### Asset discovery goes through the build-time manifest

An APK's assets live inside the archive, reachable only through the JNI
`AssetManager` — `std::fs::read_dir("assets/songs")` returns `Err`, and the
runtime scans would find nothing at all.

This is the same constraint wasm already had, so Android reuses the same
solution: `#[cfg]`-split scan functions backed by a `build.rs`-generated
manifest, now keyed on
`any(target_arch = "wasm32", target_os = "android")`.

- **Lessons were broken on wasm before this**, not just on Android.
  `lessons::catalog` had no manifest path at all, because it reads each
  `lesson.json`'s bytes directly rather than through `AssetServer`. It now
  has one (`crates/harmonicon-song/build.rs`), embedding the JSON text with
  `include_str!`. Fixing Android fixed wasm.
- **iOS is deliberately excluded.** An app bundle's Resources directory
  reads like any other, so iOS keeps the runtime scan and the
  `~/Harmonicon` drop-folder dynamism.

### The microphone permission

`RECORD_AUDIO` is "dangerous": declaring it in the manifest only makes it
requestable, and until the user grants it, opening a cpal input stream fails
indistinguishably from a broken device.

`harmonicon-audio`'s `permission` module calls
`Activity.checkSelfPermission`/`requestPermissions` over JNI.
`audio_input::start_capture` asks first and parks in
`MicStatus::AwaitingPermission` — a state that already existed as
groundwork, which the Options page already renders a banner for — and
`retry_capture_when_permission_granted` polls until the dialog is answered.
It polls because the result is delivered to an `onRequestPermissionsResult`
callback on a Java activity this codebase doesn't own, and a once-per-install
dialog doesn't justify routing that back across JNI.

Off Android, `microphone_granted()` returns `true` and
`request_microphone()` does nothing, so call sites need no `#[cfg]`.

### `android_main`, and why the root package has a library

Android never calls `main`: the platform loads a shared library and calls
`android_main`. That forced the composition root out of `src/main.rs` into
`src/lib.rs`'s `run()`, which both entry points now call.

The cdylib is its own crate rather than a second `crate-type` on the root
package because `default-members` includes every member — a cdylib on the
root would relink the whole Bevy app on every desktop `cargo build`. Instead
`harmonicon-android`'s dependency on the game is target-gated and its
`src/lib.rs` is entirely `#[cfg(target_os = "android")]`, so off Android it
is an empty cdylib with no dependencies (measured: 4.2 MB with zero Bevy
symbols, against a 517 MB desktop binary).

## Size

147 MB APK: a 108 MB stripped cdylib (stored uncompressed so it maps
straight out of the APK rather than unpacking a second copy into `/data`)
plus ~38 MB of assets. Only `arm64-v8a` is built; adding `armeabi-v7a` means
another full Rust build and roughly doubles the download. If this needs to
come down, the first target is `assets/themes` (21 MB of the 38).

## What has *not* been done

- **Never installed or launched.** See the top of this file.
- **No touch-input pass.** Keyboard-only actions already have on-screen
  equivalents, but nothing has been sized for a thumb.
- **No app icon** — the APK currently has none (`icon=''`).
- **Only arm64-v8a.**
- **Release builds are debug-signed**, so `assembleRelease` produces
  something installable without a keystore. Replace before distributing.
- **Opening the user guide does nothing.**
  `help_about::open_in_default_app` returns `Unsupported`; doing it properly
  means handing an `Intent` to the system over JNI.
- **iOS remains untouched** and needs Xcode.
