# Changelog

## v0.0.10 — 2026-08-31

- release.sh: stop --dry-run discarding a hand-edited version
- release.sh: allow a version-only dirty tree, and write a changelog
- Fix the Android CI job failing whenever the cache is restored
- Add scripts/release.sh to cut a release in one command
- Warn when the chosen detector cannot hear a chart's chords
- Pin what each pitch algorithm can actually hear of a chord
- Report a microphone unplugged mid-session instead of going quietly deaf
- Reconcile the version, and make the tag/manifest agreement a CI gate
- Correct the 1.0 unwrap-triage figure: it was a miscount
- Say so during play when the microphone isn't working
- Greet a first-time player instead of dropping them on the main menu
- Define 1.0 as a readiness gate, scoped to desktop
- Read font cmaps with skrifa, not the unmaintained ttf-parser
