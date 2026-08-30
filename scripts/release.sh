#!/usr/bin/env bash
#
# Cut a release: bump the version everywhere it is written down, commit, tag,
# and push.
#
#   scripts/release.sh --patch      0.0.10 -> 0.0.11
#   scripts/release.sh --minor      0.0.10 -> 0.1.0
#   scripts/release.sh --major      0.0.10 -> 1.0.0
#
#   --dry-run   print every change and stop, touching nothing
#
# Why a script rather than a checklist: the version is written in four places
# that are read by different things at different times, and they had already
# drifted apart once — Cargo.toml said 0.1.0 through every 0.0.x tag, and
# Android's versionName still says 0.1.0 today. `release.yaml`'s
# `check_version_matches_tag` catches a tag that disagrees with Cargo.toml,
# but nothing catches the other two, because nothing reads them until someone
# installs a package.
#
# Deliberately NOT edited here:
#   packaging/macos/Info.plist     %%VERSION%% placeholder, substituted at
#                                  build time from the tag
#   packaging/windows/harmonicon.iss
#                                  /DMyAppVersion passed in by CI from the tag
# Both derive from the tag this script creates, so writing a number into them
# would create a second source of truth for no gain.

set -euo pipefail

cd "$(dirname "$0")/.."

CARGO_TOML="Cargo.toml"
CARGO_LOCK="Cargo.lock"
GRADLE="packaging/android/app/build.gradle.kts"
METAINFO="packaging/flatpak/io.github.tcanabrava.harmonicon.metainfo.xml"
RELEASE_BRANCH="main"

die() { printf '\nrelease: %s\n' "$*" >&2; exit 1; }
step() { printf '  %s\n' "$*"; }

# ── Arguments ────────────────────────────────────────────────────────────────

BUMP=""
DRY_RUN=0
for arg in "$@"; do
    case "$arg" in
        --major|--minor|--patch)
            [ -n "$BUMP" ] && die "pick exactly one of --major/--minor/--patch"
            BUMP="${arg#--}"
            ;;
        --dry-run) DRY_RUN=1 ;;
        -h|--help) sed -n '3,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown argument '$arg' (expected --major, --minor or --patch)" ;;
    esac
done
[ -n "$BUMP" ] || die "expected one of --major, --minor, --patch (see --help)"

# ── Current version, and the one we're going to ──────────────────────────────

# The first `version =` at column 0 is the root [package]'s — crates/ members
# have their own manifests, and this is the one feeding CARGO_PKG_VERSION.
CURRENT=$(grep -m1 '^version = ' "$CARGO_TOML" | cut -d'"' -f2)
[[ "$CURRENT" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]] \
    || die "Cargo.toml version '$CURRENT' is not a three-part semver"
MAJOR="${BASH_REMATCH[1]}"; MINOR="${BASH_REMATCH[2]}"; PATCH="${BASH_REMATCH[3]}"

case "$BUMP" in
    major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
    minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
    patch) PATCH=$((PATCH + 1)) ;;
esac
NEXT="${MAJOR}.${MINOR}.${PATCH}"
TAG="v${NEXT}"

# Sort-based comparison, so this holds even if someone hand-edits a manifest
# to something lower than a tag that already exists.
higher_of() { printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1; }

[ "$(higher_of "$CURRENT" "$NEXT")" = "$NEXT" ] && [ "$CURRENT" != "$NEXT" ] \
    || die "refusing to go from $CURRENT to $NEXT"

# Every existing tag, normalised to three parts — the tag line has some
# four-component strays (v0.0.9.1) that `sort -V` would otherwise rank oddly.
HIGHEST_TAG=$(git tag \
    | sed 's/^v//' \
    | grep -E '^[0-9]+\.[0-9]+\.[0-9]+' \
    | cut -d. -f1-3 \
    | sort -V | tail -1 || true)
if [ -n "$HIGHEST_TAG" ] && [ "$(higher_of "$HIGHEST_TAG" "$NEXT")" != "$NEXT" ]; then
    die "tag $TAG would not be above the highest existing tag (v$HIGHEST_TAG)"
fi
git rev-parse -q --verify "refs/tags/$TAG" >/dev/null \
    && die "tag $TAG already exists"

# Android wants a monotonically increasing integer, separate from the name.
# This encoding stays ordered as long as minor and patch stay under 100.
[ "$MINOR" -lt 100 ] && [ "$PATCH" -lt 100 ] \
    || die "minor/patch >= 100 would break the Android versionCode encoding"
VERSION_CODE=$(( MAJOR * 10000 + MINOR * 100 + PATCH ))

TODAY=$(date +%Y-%m-%d)

printf '\nrelease %s -> %s (tag %s)\n\n' "$CURRENT" "$NEXT" "$TAG"

# ── Repository state ─────────────────────────────────────────────────────────

BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "$RELEASE_BRANCH" ] \
    || die "on branch '$BRANCH'; releases are cut from '$RELEASE_BRANCH'"
[ -z "$(git status --porcelain --untracked-files=no)" ] \
    || die "working tree has uncommitted changes; commit or stash them first"

# Only for a real release: a preview shouldn't need the network, and being
# behind origin only matters for something about to be pushed.
if [ "$DRY_RUN" -eq 0 ]; then
    git fetch --quiet origin "$RELEASE_BRANCH" || die "could not reach origin"
    BEHIND=$(git rev-list --count "HEAD..origin/$RELEASE_BRANCH")
    [ "$BEHIND" -eq 0 ] \
        || die "$BEHIND commit(s) behind origin/$RELEASE_BRANCH; pull first"
fi

# ── Edits ────────────────────────────────────────────────────────────────────

apply_edits() {
    # Root [package] only: `0,/re/` bounds the substitution to the first match.
    sed -i "0,/^version = \"$CURRENT\"/s//version = \"$NEXT\"/" "$CARGO_TOML"
    step "Cargo.toml            version = \"$NEXT\""

    # The lock carries the workspace's own package version too. Rewritten by
    # cargo rather than by hand so the rest of the file stays exactly as
    # cargo would write it.
    cargo metadata --offline --format-version 1 >/dev/null 2>&1 \
        || die "cargo could not refresh $CARGO_LOCK"
    step "Cargo.lock            refreshed by cargo"

    sed -i "s/^\( *versionName = \)\".*\"/\1\"$NEXT\"/" "$GRADLE"
    sed -i "s/^\( *versionCode = \).*/\1$VERSION_CODE/" "$GRADLE"
    step "$GRADLE"
    step "                      versionName \"$NEXT\", versionCode $VERSION_CODE"

    # Newest first: AppStream expects the list in descending order, and
    # Flathub renders the top entry as "what's new".
    sed -i "s|^\( *\)<releases>|\1<releases>\n\1  <release version=\"$NEXT\" date=\"$TODAY\" />|" \
        "$METAINFO"
    step "$METAINFO"
    step "                      <release version=\"$NEXT\" date=\"$TODAY\" />"
}

if [ "$DRY_RUN" -eq 1 ]; then
    echo "would change:"
    apply_edits
    echo
    echo "would then: commit, tag $TAG, and push both to origin/$RELEASE_BRANCH"
    echo "reverting the edits (--dry-run)"
    # Safe to discard rather than stash: the clean-tree check above already
    # refused to get this far if any of these four had uncommitted work in
    # them, so the only changes being thrown away are the ones just made.
    git checkout -- "$CARGO_TOML" "$CARGO_LOCK" "$GRADLE" "$METAINFO"
    exit 0
fi

echo "changing:"
apply_edits

# Cheap sanity check that the manifest still parses and the version took.
WRITTEN=$(cargo metadata --offline --format-version 1 --no-deps \
    | grep -o "\"name\":\"harmonicon\",\"version\":\"[^\"]*\"" \
    | head -1 | sed 's/.*"version":"\([^"]*\)".*/\1/')
[ "$WRITTEN" = "$NEXT" ] \
    || die "cargo reports version '$WRITTEN' after the edit, expected '$NEXT'"

# ── Commit, tag, push ────────────────────────────────────────────────────────

echo
git --no-pager diff --stat -- "$CARGO_TOML" "$CARGO_LOCK" "$GRADLE" "$METAINFO"
echo
read -r -p "Commit, tag $TAG, and push to origin/$RELEASE_BRANCH? [y/N] " reply
case "$reply" in
    [yY]|[yY][eE][sS]) ;;
    *) echo "aborted; edits left in the working tree for inspection"; exit 1 ;;
esac

git add "$CARGO_TOML" "$CARGO_LOCK" "$GRADLE" "$METAINFO"
git commit -q -m "Release $TAG"
# Annotated: `git describe` and the release workflow both prefer a real tag
# object over a lightweight ref.
git tag -a "$TAG" -m "Release $TAG"

git push origin "$RELEASE_BRANCH"
git push origin "$TAG"

printf '\nreleased %s\n' "$TAG"
echo "  the tag push starts .github/workflows/release.yaml, which re-checks"
echo "  the tag against Cargo.toml before building anything."
