// SPDX-License-Identifier: MIT

//! `android_main`, the Android entry point.
//!
//! Android never calls a `main`: the activity loads this shared library and
//! calls `android_main`, handing over the `AndroidApp` that owns the event
//! loop and the JNI handles. Bevy reads that back out of a global
//! (`bevy::android::ANDROID_APP`) when its winit backend starts, so it has to
//! be stashed there *before* the app runs.
//!
//! This is `#[bevy_main]`'s expansion, written out. The macro insists on
//! being applied to a function literally named `main`, which would mean a
//! stray `pub fn main` in a library and a dead one on every other target;
//! spelling it out costs four lines and says plainly what happens.

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(android_app);
    harmonicon::run();
}
