# scripts/

## `release.sh` — cut a release

```bash
./scripts/release.sh --patch          # 0.0.10 -> 0.0.11
./scripts/release.sh --minor          # 0.0.10 -> 0.1.0
./scripts/release.sh --major          # 0.0.10 -> 1.0.0
./scripts/release.sh --patch --dry-run   # print every change and stop
```

Exactly one of `--major`/`--minor`/`--patch`. It bumps the version
everywhere it's written down, commits, tags, and pushes — asking before the
push, and printing a diffstat first.

### What it writes, and what it deliberately doesn't

| File | Why |
|---|---|
| `Cargo.toml` | the root `[package]` version — this is `CARGO_PKG_VERSION`, what Help / About shows |
| `Cargo.lock` | carries the workspace's own package version; refreshed by `cargo metadata`, not by hand |
| `packaging/android/app/build.gradle.kts` | `versionName` **and** `versionCode` |
| `packaging/flatpak/*.metainfo.xml` | prepends a `<release>` entry dated today; Flathub reads this |
| `CHANGELOG.md` | prepends a section for the new tag, one bullet per commit subject since the last tag |

**Not** touched, on purpose:

- `packaging/macos/Info.plist` — a `%%VERSION%%` placeholder that CI
  substitutes from the tag.
- `packaging/windows/harmonicon.iss` — CI passes `/DMyAppVersion` from the
  tag.

Both already derive from the tag this script creates. Writing a number into
them would add a second source of truth for no gain.

### Why a script instead of a checklist

These numbers are read by different things at different times, so they drift
silently and only disagree in front of a user. They already had: `Cargo.toml`
said `0.1.0` through every `0.0.x` tag, so the version shown inside the game
was one no release ever carried — and Android's `versionName` was still
`0.1.0` when this script was written, because nothing reads it until someone
installs an APK.

`release.yaml`'s `check_version_matches_tag` catches a tag that disagrees
with `Cargo.toml`, but nothing catches the packaging files.

### The changelog

Each release prepends a section to `CHANGELOG.md`, newest first, built from
`git log --no-merges` since the last tag — one bullet per commit subject.

Subjects rather than a hand-written summary because this project's commit
messages already lead with a real sentence about what changed. The useful
changelog is sitting in them, and anything maintained alongside would be a
second thing to keep true.

### A dirty tree, when it's only the version

Bumping often starts by hand — editing `Cargo.toml` to line the manifest back
up with the tags, which also touches `Cargo.lock`. Refusing that would be
refusing the normal way in, so the script allows it and sweeps it into the
release commit.

The exemption is narrow on purpose. Only those two files may be modified,
**and** every changed line in them must itself be a `version = "x.y.z"` line
— a dependency bump in `Cargo.lock` is not a version-only change. Anything
else is still refused, because an unrelated half-finished edit must not ride
along in a commit called `Release vX.Y.Z`.

### Refusals

It stops rather than doing something surprising when:

- any *other* uncommitted change is present (see above);
- the new version wouldn't be **above** the current one, or above the
  highest existing tag (comparison is `sort -V`, and the tag line's
  four-component strays like `v0.0.9.1` are normalised to three parts);
- the tag already exists;
- you're not on `main`;
- the branch is behind `origin/main`;
- `minor` or `patch` would reach 100, which would break the Android
  `versionCode` encoding (`major*10000 + minor*100 + patch` — chosen because
  Play Store requires a monotonically increasing integer, separate from the
  name).

### After it pushes

The tag push starts `.github/workflows/release.yaml`, whose first job
re-checks the tag against `Cargo.toml` before anything is built.

## Everything else here

`generate_lesson_files.py`, `create_base_harp_model.py`, `make_note_cube.py`
and the `.glb` models are one-off authoring/asset tools, run by hand.
`git-hooks/` holds the hooks and `install.sh`; run that once per clone.
