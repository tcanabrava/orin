// Android packaging lives here rather than beside the Rust crate, matching
// packaging/{flatpak,macos,windows}. The Rust side is
// `crates/harmonicon-android` (a cdylib exporting `android_main`); this
// project only compiles it via cargo-ndk and wraps the result in an APK.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.FAIL_ON_PROJECT_REPOS
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "harmonicon"
include(":app")
