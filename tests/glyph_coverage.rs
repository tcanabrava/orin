// SPDX-License-Identifier: MIT

//! Every character the game draws must exist in one of the bundled fonts.
//!
//! A character missing from all of them renders as a **tofu box** — a silent,
//! runtime-only failure. It compiles, it passes every other test, and the
//! only way to notice is to look at a rendered frame in the right language.
//! Five shipped that way in all three locales (`🎹`, `⏳`, `★`, `⚠`, `🗣`)
//! and went unnoticed until an emulator screenshot happened to show one.
//!
//! `dialogs::font_fallback` bundles three subsetted fonts for characters
//! `FreeSans.otf` lacks, and keeps a hand-maintained list of what each one
//! covers. That list is *documentation of intent*: adding a character to it
//! without also re-subsetting the `.ttf` leaves the glyph just as missing.
//! So this test deliberately reads the **font binaries**, not the Rust
//! lists — the fonts are the only thing that decides what actually draws.
//!
//! See the "add a rule, add a check" table in `CLAUDE.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Below this, everything is plain ASCII/Latin that `FreeSans` covers
/// comprehensively; checking it would only add noise. The interesting range
/// is symbols, arrows, dingbats and emoji.
const FIRST_INTERESTING: u32 = 0x2000;

/// Invisible formatting characters, which have no glyph *by design* — asking
/// whether a font covers them is the wrong question.
///
/// `localization::strip_bidi_isolates` names U+2068/U+2069 explicitly (Fluent
/// wraps interpolated arguments in them), so they appear in source as
/// characters being *removed*, never drawn.
fn is_invisible_format_char(cp: u32) -> bool {
    matches!(cp,
        0x200B..=0x200F  // zero-width space/non-joiner/joiner, LRM, RLM
        | 0x202A..=0x202E  // bidi embedding and override
        | 0x2060..=0x2064  // word joiner, invisible operators
        | 0x2066..=0x2069  // bidi isolates (Fluent's FSI/PDI)
        | 0xFEFF           // zero-width no-break space / BOM
    )
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every codepoint a font's `cmap` can actually render.
fn cmap_codepoints(path: &Path) -> BTreeSet<u32> {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let face = ttf_parser::Face::parse(&data, 0)
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let mut out = BTreeSet::new();
    for subtable in face.tables().cmap.expect("font has no cmap").subtables {
        // Non-unicode subtables (old Mac encodings) would report codepoints
        // in a different space entirely, so they'd be actively misleading.
        if subtable.is_unicode() {
            subtable.codepoints(|cp| {
                out.insert(cp);
            });
        }
    }
    out
}

/// The union of everything the bundled fonts can draw: `FreeSans` plus the
/// three subsetted fallbacks `font_fallback` splits text runs across.
fn bundled_coverage() -> BTreeSet<u32> {
    let fonts = repo_root().join("assets/fonts");
    // Bravura is in here because `music_score::notation` renders SMuFL
    // private-use codepoints (U+E0xx-U+E2xx) with it — those are real,
    // deliberate glyphs, not tofu, even though no text font has them.
    [
        "FreeSans.otf",
        "Bravura.otf",
        "fallback_emoji.ttf",
        "fallback_symbols.ttf",
        "fallback_arrows.ttf",
    ]
    .iter()
    .flat_map(|name| cmap_codepoints(&fonts.join(name)))
    .collect()
}

/// Formats offenders as one line each, so a failure names the character,
/// its codepoint, and somewhere it's used.
fn describe(missing: &BTreeMap<u32, BTreeSet<String>>) -> String {
    missing
        .iter()
        .map(|(cp, where_)| {
            let sites: Vec<&str> = where_.iter().take(3).map(String::as_str).collect();
            let ch = char::from_u32(*cp).unwrap_or('?');
            format!("  U+{cp:04X} {ch:?} — used in {}", sites.join(", "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const REMEDY: &str = "\n\nEach of these renders as an empty box. Either use a character \
FreeSans.otf already has (check with the same cmap this test reads), or add \
the codepoint to the matching list in dialogs::font_fallback AND re-subset \
the corresponding assets/fonts/fallback_*.ttf — the list alone changes \
nothing.";

/// Walks `dir` recursively, calling `visit` for every file whose extension
/// matches `ext`.
fn walk(dir: &Path, ext: &str, visit: &mut impl FnMut(&Path, String)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, ext, visit);
        } else if path.extension().is_some_and(|e| e == ext)
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            visit(&path, text);
        }
    }
}

#[test]
fn every_glyph_in_a_locale_string_is_in_a_bundled_font() {
    let covered = bundled_coverage();
    let mut missing: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();

    walk(
        &repo_root().join("assets/locales"),
        "ftl",
        &mut |path, text| {
            // A locale file is `key = value` lines plus comments; only the value
            // is ever drawn.
            for line in text.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    continue;
                }
                let value = match line.split_once('=') {
                    Some((_, v)) => v,
                    // Continuation lines of a multi-line message are drawn too.
                    None if trimmed.is_empty() => continue,
                    None => line,
                };
                for cp in value.chars().map(u32::from) {
                    if cp >= FIRST_INTERESTING
                        && !is_invisible_format_char(cp)
                        && !covered.contains(&cp)
                    {
                        let locale = path
                            .components()
                            .nth_back(2)
                            .map(|c| c.as_os_str().to_string_lossy().into_owned())
                            .unwrap_or_default();
                        missing.entry(cp).or_default().insert(locale);
                    }
                }
            }
        },
    );

    assert!(
        missing.is_empty(),
        "locale strings use {} character(s) no bundled font can draw:\n{}{REMEDY}",
        missing.len(),
        describe(&missing)
    );
}

#[test]
fn every_glyph_escape_in_source_is_in_a_bundled_font() {
    let covered = bundled_coverage();
    let escape = regex::Regex::new(r"\\u\{([0-9A-Fa-f]{2,6})\}").expect("valid regex");
    let mut missing: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();

    // Button icons and similar are written as `\u{2191}` escapes in source
    // rather than pasted literally, so scanning the locales alone would miss
    // every one of them.
    let mut roots = vec![repo_root().join("src")];
    if let Ok(crates) = std::fs::read_dir(repo_root().join("crates")) {
        roots.extend(crates.flatten().map(|e| e.path().join("src")));
    }

    for root in roots {
        walk(&root, "rs", &mut |path, text| {
            for caps in escape.captures_iter(&text) {
                let Ok(cp) = u32::from_str_radix(&caps[1], 16) else {
                    continue;
                };
                if cp >= FIRST_INTERESTING
                    && !is_invisible_format_char(cp)
                    && !covered.contains(&cp)
                {
                    let file = path
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    missing.entry(cp).or_default().insert(file);
                }
            }
        });
    }

    assert!(
        missing.is_empty(),
        "source uses {} glyph escape(s) no bundled font can draw:\n{}{REMEDY}",
        missing.len(),
        describe(&missing)
    );
}

/// Guards the test itself: if the fonts moved or failed to parse, both
/// checks above would pass vacuously against an empty coverage set.
#[test]
fn the_bundled_fonts_parse_and_cover_something() {
    let covered = bundled_coverage();
    assert!(
        covered.len() > 1000,
        "only {} codepoints across the bundled fonts — did a font move or fail to parse? \
         The coverage tests would silently pass against an empty set.",
        covered.len()
    );
    // Spot-check one character from each fallback, so a missing *file* is
    // caught rather than quietly reducing coverage.
    for (name, cp) in [
        ("emoji 🎵", 0x1F3B5),
        ("symbols ✓", 0x2713),
        ("arrows ↶", 0x21B6),
    ] {
        assert!(
            covered.contains(&cp),
            "{name} (U+{cp:04X}) missing — is assets/fonts/fallback_*.ttf still there?"
        );
    }
}
