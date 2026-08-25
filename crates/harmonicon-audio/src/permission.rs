// SPDX-License-Identifier: MIT

//! Microphone permission, where the platform has one.
//!
//! Two functions with the same names on every target: [`microphone_granted`]
//! and [`request_microphone`]. Desktop has no runtime permission model, so
//! there it answers "granted" and requesting is a no-op — the whole flow in
//! [`audio_input`](crate::audio_input) is then written once, unconditionally,
//! with no `#[cfg]` at the call site.
//!
//! Android is the target that actually needs it: `RECORD_AUDIO` is a
//! "dangerous" permission, so since API 23 it must be granted by the user at
//! runtime, and until it is, opening a cpal input stream fails. That failure
//! is indistinguishable from a broken device, which is why
//! [`MicStatus::AwaitingPermission`](crate::audio_input::MicStatus) exists
//! separately from `Failed`.

/// Whether the app may open the microphone right now.
///
/// Always `true` off Android: desktop grants access at the OS level (or the
/// stream simply fails), and there is no API to consult.
#[cfg(not(target_os = "android"))]
pub fn microphone_granted() -> bool {
    true
}

/// No-op off Android — nothing to ask for.
#[cfg(not(target_os = "android"))]
pub fn request_microphone() {}

/// `Activity.checkSelfPermission(RECORD_AUDIO) == PERMISSION_GRANTED`.
///
/// Any JNI error is reported as *not* granted: the honest answer when the
/// check itself couldn't run is "don't assume we may record".
#[cfg(target_os = "android")]
pub fn microphone_granted() -> bool {
    match with_activity(|env, activity| {
        let permission = env.new_string(RECORD_AUDIO)?;
        let result = env
            .call_method(
                activity,
                "checkSelfPermission",
                "(Ljava/lang/String;)I",
                &[(&permission).into()],
            )?
            .i()?;
        // android.content.pm.PackageManager.PERMISSION_GRANTED
        Ok(result == 0)
    }) {
        Ok(granted) => granted,
        Err(err) => {
            bevy::log::warn!("Could not check RECORD_AUDIO permission: {err}");
            false
        }
    }
}

/// Shows the system permission dialog, if it hasn't already been answered.
///
/// Fire-and-forget: the result arrives on the Java side, in an
/// `onRequestPermissionsResult` callback belonging to an activity this crate
/// doesn't own. Rather than route that back across JNI,
/// [`audio_input::retry_capture_when_permission_granted`](crate::audio_input::
/// retry_capture_when_permission_granted) polls [`microphone_granted`] — the
/// dialog is a once-per-install event, so a cheap poll while it's open costs
/// nothing worth optimising.
#[cfg(target_os = "android")]
pub fn request_microphone() {
    if let Err(err) = with_activity(|env, activity| {
        let permission = env.new_string(RECORD_AUDIO)?;
        let permissions =
            env.new_object_array(1, "java/lang/String", jni::objects::JObject::null())?;
        env.set_object_array_element(&permissions, 0, &permission)?;
        env.call_method(
            activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[(&permissions).into(), REQUEST_CODE.into()],
        )?;
        Ok(())
    }) {
        bevy::log::warn!("Could not request RECORD_AUDIO permission: {err}");
    }
}

#[cfg(target_os = "android")]
const RECORD_AUDIO: &str = "android.permission.RECORD_AUDIO";

/// Passed to `requestPermissions` and handed back to the activity's
/// `onRequestPermissionsResult`. Nothing here reads it — see
/// [`request_microphone`] — but the API requires one.
#[cfg(target_os = "android")]
const REQUEST_CODE: i32 = 0;

/// Runs `f` with a JNI environment attached to the current thread and the
/// app's `Activity` object.
///
/// The VM and activity pointers come from `ndk_context`, which
/// `android-activity` populates when it hands us `AndroidApp` — the same
/// globals every other NDK crate reads, rather than a second copy of that
/// state threaded through from `android_main`.
#[cfg(target_os = "android")]
fn with_activity<T>(
    f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> jni::errors::Result<T>,
) -> jni::errors::Result<T> {
    let ctx = ndk_context::android_context();
    // SAFETY: `ndk_context` hands back the JavaVM and Activity that
    // `android-activity` registered at startup; both outlive the app.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };
    let mut env = vm.attach_current_thread()?;
    f(&mut env, &activity)
}
