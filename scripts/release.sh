#!/bin/bash
# Release partkeepr-ng to the spectraq Gentoo overlay (gitolite-first).
#
# Pre-flight ritual + publish. Version is read from backend/Cargo.toml (the same
# source make-dist.sh uses). Publishing commits + tags in gitolite and promotes
# the distribution bare, via scripts/overlay-release.sh.
#
# Usage:
#   ./scripts/release.sh          # release at current backend/Cargo.toml version
#   ./scripts/release.sh --check  # pre-flight only; no publish
set -euo pipefail
cd "$(dirname "$0")/.."

PACKAGE="partkeepr-ng"
VERSION=$(grep '^version' backend/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
EBUILD="ebuild/${PACKAGE}-${VERSION}.ebuild"

CHECK_ONLY=0
for arg in "$@"; do
    case "$arg" in
        --check) CHECK_ONLY=1 ;;
        --help|-h) echo "Usage: $0 [--check]"; exit 0 ;;
        *) echo "Unknown arg: $arg" >&2; exit 1 ;;
    esac
done

echo "=== Releasing ${PACKAGE}-${VERSION} ==="

# Shared release metadata must agree before build, tagging, or publishing.
python3 scripts/version.py --check

# 1. Clean working tree.
echo "[pre-flight 1/6] Working tree clean..."
git diff-index --quiet HEAD -- || { echo "  FAIL: uncommitted changes." >&2; git status --short >&2; exit 1; }
[ -z "$(git ls-files --others --exclude-standard)" ] || { echo "  FAIL: untracked files." >&2; exit 1; }
echo "  OK."

# 2. On master.
echo "[pre-flight 2/6] On master..."
[ "$(git rev-parse --abbrev-ref HEAD)" = master ] || { echo "  FAIL: not on master." >&2; exit 1; }
echo "  OK."

# 3. Ebuild for this version exists.
echo "[pre-flight 3/6] ${EBUILD} exists..."
[ -f "$EBUILD" ] || { echo "  FAIL: $EBUILD missing (bump backend/Cargo.toml AND copy the prior ebuild)." >&2; exit 1; }
echo "  OK."

# 4. Cargo.lock in sync with Cargo.toml.
echo "[pre-flight 4/6] Cargo.lock in sync..."
cargo metadata --quiet --locked --format-version 1 >/dev/null 2>&1 || { echo "  FAIL: Cargo.lock stale; run 'cargo build' and commit." >&2; exit 1; }
echo "  OK."

# 5. Release build compiles. partkeepr backend tests need a live database, so the
#    gate here is a clean release build rather than `cargo test`.
echo "[pre-flight 5/6] Release build compiles..."
cargo build --release --locked --quiet || { echo "  FAIL: cargo build --release." >&2; exit 1; }
echo "  OK."

# 6. Ebuild references every ebuild/files/* entry (companion file added without
#    its doins/newins/doexe, or a wrong prior ebuild copied as the base).
echo "[pre-flight 6/6] Ebuild references all ebuild/files/*..."
if [ -d ebuild/files ]; then
    missing=()
    for f in ebuild/files/*; do
        [ -e "$f" ] || continue
        bn=$(basename "$f")
        grep -q "$bn" "$EBUILD" || missing+=("$bn")
    done
    if [ ${#missing[@]} -gt 0 ]; then
        echo "  FAIL: $EBUILD does not reference these ebuild/files/ entries:" >&2
        printf '    - %s\n' "${missing[@]}" >&2
        exit 1
    fi
fi
echo "  OK."

if [ "$CHECK_ONLY" -eq 1 ]; then
    echo "=== pre-flight passed (--check); not publishing ==="
    exit 0
fi

# --- Publish ---
echo "[1/3] Building source tarball..."
./scripts/make-dist.sh

# The v<version> tag on this repo is now just a source-release marker -- it no
# longer triggers an overlay release (that path was retired). The overlay
# publish is the explicit overlay-release.sh call below.
echo "[2/3] Tagging source release + pushing project repo..."
git push origin master
TAG="v${VERSION}"
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "  $TAG already exists; not re-tagging."
else
    git tag -a "$TAG" -m "Release ${PACKAGE} ${VERSION}"
    git push origin "$TAG"
fi

echo "[3/3] Publishing to overlay (gitolite-first)..."
./scripts/overlay-release.sh "${PACKAGE}" "${VERSION}" "/tmp/${PACKAGE}-${VERSION}.tar.xz" ./ebuild

echo ""
echo "=== Released ${PACKAGE}-${VERSION} ==="
echo "Clients: sudo emaint sync -r spectraq && emerge -av app-spectraq/${PACKAGE}"
