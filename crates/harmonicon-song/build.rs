// SPDX-License-Identifier: MIT

//! Generates the bundled-lesson manifest that `lessons::catalog` includes on
//! targets whose `assets/` tree isn't a readable local directory — wasm (no
//! filesystem; the asset reader talks HTTP) and Android (assets live inside
//! the APK, reachable only through the JNI `AssetManager`). iOS is *not* one
//! of these: an app bundle's Resources directory reads like any other, so it
//! keeps the runtime scan.
//!
//! Same reasoning as `harmonicon-platform`'s build script — and the same
//! reason it can't live there: `include!(concat!(env!("OUT_DIR"), ...))`
//! reads the *including* crate's own OUT_DIR, and OUT_DIR is per-package.
//!
//! Unlike the song/theme manifests, which only need *names* (their contents
//! then load through the asset server), a lesson is discovered by reading
//! `lesson.json` itself — `scan_lessons_root` parses those bytes directly
//! rather than going through `AssetServer`. So this manifest embeds the JSON
//! text with `include_str!`, not just the directory names.

use std::path::Path;

fn main() {
    generate_bundled_lesson_manifest();
}

/// Writes `$OUT_DIR/lesson_manifest.rs`. A no-op (two env var reads) unless
/// the crate is being built for a target that needs it, so desktop builds
/// pay nothing and keep scanning `assets/lessons` for real at runtime.
fn generate_bundled_lesson_manifest() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH");
    let os = std::env::var("CARGO_CFG_TARGET_OS");
    if arch.as_deref() != Ok("wasm32") && os.as_deref() != Ok("android") {
        return;
    }

    println!("cargo:rerun-if-changed=../../assets/lessons");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set by cargo");
    let dest = Path::new(&out_dir).join("lesson_manifest.rs");

    // `include_str!` in the generated file resolves relative to that file,
    // which lives in OUT_DIR — so the paths it embeds have to be absolute.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set by cargo");
    let lessons_root = Path::new(&manifest_dir).join("../../assets/lessons");

    let mut out = String::from("// Auto-generated at build time by build.rs — do not edit.\n");
    out.push_str(
        "/// `(unit_dir, lesson_dir, lesson.json contents)`, in curriculum order.\n\
         pub const BUNDLED_LESSONS: &[(&str, &str, &str)] = &[\n",
    );

    for (unit, lesson, path) in scan_lessons_for_manifest(&lessons_root) {
        out.push_str(&format!(
            "    ({unit:?}, {lesson:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    out.push_str("];\n");

    std::fs::write(&dest, out).expect("failed to write lesson manifest");
}

/// Mirrors `lessons::catalog::scan_lessons_root`'s discovery rule exactly —
/// `<unit_dir>/<lesson_dir>/lesson.json`, both levels sorted by directory
/// name so the `01_`/`02_` prefixes give the curriculum order — so the two
/// implementations can't drift.
fn scan_lessons_for_manifest(root: &Path) -> Vec<(String, String, std::path::PathBuf)> {
    let mut found = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return found;
    };
    let mut unit_dirs: Vec<_> = rd
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    unit_dirs.sort();

    for unit_dir in unit_dirs {
        let Ok(rd) = std::fs::read_dir(&unit_dir) else {
            continue;
        };
        let mut lesson_dirs: Vec<_> = rd
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .collect();
        lesson_dirs.sort();

        for lesson_dir in lesson_dirs {
            let manifest = lesson_dir.join("lesson.json");
            if !manifest.is_file() {
                continue; // not a lesson dir
            }
            let (Some(unit), Some(lesson)) = (
                unit_dir.file_name().and_then(|n| n.to_str()),
                lesson_dir.file_name().and_then(|n| n.to_str()),
            ) else {
                continue;
            };
            found.push((unit.to_string(), lesson.to_string(), manifest));
        }
    }
    found
}
