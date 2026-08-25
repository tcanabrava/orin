// SPDX-License-Identifier: MIT

//! Desktop entry point. The app itself is assembled in `lib.rs`'s `run()`,
//! which Android's `android_main` also calls — see `crates/harmonicon-android`.

fn main() {
    harmonicon::run();
}
