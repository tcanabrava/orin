# Android

## Status: runs on an emulator; never run on real hardware

**Verified, by actually running it** on an Android 15 (API 35) x86_64
emulator:

- It launches, and the main menu renders correctly — background art, fonts,
  all four buttons.
- **Assets load out of the APK.** That background is a theme asset, so the
  build-time manifest path works in a real `AssetManager` environment.
- **The permission flow works end to end**: the system `GrantPermissionsActivity`
  dialog appears, and once the permission is granted the polling retry picks
  it up and opens a capture stream —
  `Input device : Default Device / Sample rate : 44100 Hz | channels: 2 |
  format: F32`.
- The APK's contents were also inspected statically: cdylib exporting
  `android_main` and `GameActivity_onCreate`, `GameActivity` in
  `classes.dex`, `RECORD_AUDIO` declared, 186 asset entries, `debug_songs`
  excluded.

CI's `android_check` job type-checks the target on every push.

**Not verified.** No real device, so: **nobody has played a harmonica into
it.** An emulator opening a capture stream says the plumbing is connected; it
says nothing about latency, gain, or whether pitch detection works against a
phone mic's AGC and noise suppression — which for this game is the whole
product. Also unknown: touch target sizes, whether landscape-only is right,
real frame rates, and whether the Song Editor is usable on a phone at all.

Two bugs were found *only* by running it, both since fixed — see
"Two runtime-only failures" below. Neither was visible at build time.

## Known gap: nothing persists

`dirs::config_dir()` returns `None` on Android, so `settings.rs` logs *"No
config directory available; settings not saved"* and `profile.rs` does the
same silently. **Lesson progress, best scores and options are all lost on
exit.** That makes the Android build effectively demo-only until fixed.

The fix is not hard — `AndroidApp::internal_data_path()` gives the app's
private directory — but it means threading a platform-supplied path into
`settings`/`profile`, which currently derive their own location from `dirs`.

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

### Other ABIs, and the emulator

Only `arm64-v8a` is built by default: every phone worth targeting is arm64,
and each extra ABI costs another full Rust build plus ~108 MB of APK. A
desktop emulator is x86_64, so it needs an override:

```bash
./gradlew installRelease -Pharmonicon.abis=x86_64
```

Comma-separate for a fat APK (`-Pharmonicon.abis=arm64-v8a,x86_64`).

## Two runtime-only failures

Both compiled cleanly, packaged cleanly, and passed every static check on the
APK. Only launching it found them. This is the argument for keeping an
emulator in the loop.

### `ClassNotFoundException: com.google.androidgamesdk.GameActivity`

The class *was* in `classes.dex` — the real cause was hidden in a
**suppressed** exception: `NoClassDefFoundError: androidx/appcompat/app/AppCompatActivity`.
`GameActivity` extends `AppCompatActivity`, but the
`games-activity-4.4.0.pom` declares **no dependencies at all**, so appcompat
was never pulled in transitively. It has to be an explicit
`implementation("androidx.appcompat:appcompat:...")`.

That also forces the theme: `AppCompatActivity` refuses to start under a
plain platform theme, so `@android:style/Theme.NoTitleBar.Fullscreen` had to
become a `Theme.AppCompat.NoActionBar` descendant
(`res/values/themes.xml`).

### `NoSuchMethodError: Landroid/app/Application;.requestPermissions`

`ndk_context::android_context()` is the obvious way to reach the app's
context from Rust, and it is **wrong for this**: `android-activity` registers
the **`Application`** there, not the `Activity` (its `init.rs`,
`initialize_android_context(vm, app_global)`).

`Application` is a `Context`, so `checkSelfPermission` resolves on it and
appears to work — but `requestPermissions` is declared on `Activity`, so it
threw. The Activity has to come from `AndroidApp::activity_as_ptr`, which
Bevy keeps in `ANDROID_APP`.

The failure also cascaded: a throwing JNI call leaves the exception *pending*
on that thread, so every later call failed with the same opaque "Java
exception was thrown" — one bad call per frame, and the real cause never
shown. `with_activity` now calls `exception_describe`/`exception_clear`,
which is what surfaced the actual `NoSuchMethodError` in logcat.

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
