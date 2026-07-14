#!/bin/bash
# Release a spectraq package, gitolite-first.
#
# gitolite spectraq-overlay is the SOURCE OF TRUTH. This commits + tags the
# release there, then promotes the HTTPS distribution bare (what hosts emerge
# from) to that tag. Nothing reaches clients except the final promote.
#
# The version is keyed off the code, not a parallel identifier: the project's
# release.sh reads it from Cargo.toml / src/version.h and passes it here, where
# it must match the ebuild's PV. The tag <pkg>-<version> is that same value.
#
# Doctrine: gitolite master is the RELEASE LINE. Do exploratory/WIP ebuild work
# on branches -- promoting any release tag ships ALL of master up to that commit
# (a monolithic overlay can't publish one package's version in isolation).
#
# For SpectraQ packages with a source tarball served from FILES_DIR. Third-party
# / upstream-sourced ebuilds (SRC_URI off-site, e.g. qucs-s) are committed +
# tagged in gitolite by hand, then shipped with promote-to-bare.sh directly.
#
# Recovery: if this aborts AFTER step 4 (gitolite push), the release is already
# the source of truth -- finish distribution by hand: scp the tarball to
# FILES_DIR, then ./promote-to-bare.sh <pkg>-<version>.
#
# Usage: overlay-release.sh <package> <version> <tarball> <ebuild-dir> [category]
set -euo pipefail

if [ $# -lt 4 ]; then
    echo "Usage: $0 <package> <version> <tarball> <ebuild-dir> [category]" >&2
    echo "  package    - package name (e.g. sqgpsd-ng)" >&2
    echo "  version    - version from the code (Cargo.toml / version.h); must match ebuild PV" >&2
    echo "  tarball    - path to source tarball, named <package>-<version>.<ext>" >&2
    echo "  ebuild-dir - dir with the ebuild, metadata.xml, files/" >&2
    echo "  category   - overlay category (default: app-spectraq)" >&2
    exit 1
fi

PACKAGE="$1"
VERSION="$2"
TARBALL="$3"
EBUILD_DIR="$4"
CATEGORY="${5:-app-spectraq}"

# --- Configuration ---
REMOTE_HOST="chrisbuck"
GITOLITE_URL="repos@sqgit.spectraq.com:spectraq-overlay"   # source of truth (SSH/ACL)
FILES_DIR="/var/www/com.spectraq.sqgit/htdocs/files"       # distfiles served to clients
# Persistent authoring clone, refreshed each run so we don't re-clone the tree.
# This is a workspace, NOT a portage repo and NOT the chrisbuck working copy.
CLONE_DIR="${SPECTRAQ_OVERLAY_CLONE:-$HOME/.cache/spectraq-overlay}"
GIT_ID=(-c user.name=dbishop -c user.email=dbishop@spectraq.com)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EBUILD="${PACKAGE}-${VERSION}.ebuild"
TAG="${PACKAGE}-${VERSION}"
PKG_REL="${CATEGORY}/${PACKAGE}"
TARBALL_BN="$(basename "$TARBALL")"

# --- Validate inputs ---
[ -f "$TARBALL" ] || { echo "ERROR: tarball not found: $TARBALL" >&2; exit 1; }
[ -f "${EBUILD_DIR}/${EBUILD}" ] || { echo "ERROR: ebuild not found: ${EBUILD_DIR}/${EBUILD}" >&2; exit 1; }

# The Manifest is generated from the local tarball, matched by SRC_URI distfile
# name. A mismatched name makes `ebuild manifest` try to fetch from sqgit (needs
# a netrc dbishop lacks) and fail confusingly -- catch it here.
case "$TARBALL_BN" in
    "${PACKAGE}-${VERSION}."*) ;;
    *) echo "ERROR: tarball must be named ${PACKAGE}-${VERSION}.<ext> (got ${TARBALL_BN})" >&2; exit 1 ;;
esac

# Every file under ebuild/files/ must be referenced by the ebuild. Catches a
# companion file added without its doins/newins/doexe, or a wrong prior ebuild
# copied as a starting point that dropped install rules we already shipped.
if [ -d "${EBUILD_DIR}/files" ]; then
    missing=()
    for f in "${EBUILD_DIR}/files"/*; do
        [ -e "$f" ] || continue
        bn=$(basename "$f")
        grep -q "$bn" "${EBUILD_DIR}/${EBUILD}" || missing+=("$bn")
    done
    if [ ${#missing[@]} -gt 0 ]; then
        echo "ERROR: ${EBUILD_DIR}/${EBUILD} does not reference these files/ entries:" >&2
        printf '  - %s\n' "${missing[@]}" >&2
        echo "Add the doins/newins/doexe, OR rm the file if obsolete." >&2
        exit 1
    fi
fi

echo "=== Releasing ${PKG_REL}-${VERSION} (gitolite-first) ==="

# --- Step 1: refresh the local gitolite clone (authoring workspace) ---
echo "[1/6] Syncing gitolite clone..."
if [ -d "${CLONE_DIR}/.git" ]; then
    origin_url="$(git -C "$CLONE_DIR" remote get-url origin 2>/dev/null || echo "")"
    if [ "$origin_url" != "$GITOLITE_URL" ]; then
        echo "ERROR: ${CLONE_DIR} origin is '${origin_url}', not ${GITOLITE_URL}." >&2
        echo "       Remove it or point \$SPECTRAQ_OVERLAY_CLONE at a clean path." >&2
        exit 1
    fi
    git -C "$CLONE_DIR" fetch -q origin
    git -C "$CLONE_DIR" checkout -q master
    git -C "$CLONE_DIR" reset -q --hard origin/master
else
    git clone -q "$GITOLITE_URL" "$CLONE_DIR"
fi

# Refuse to re-use a version that was already released -- the tag is the key.
if git -C "$CLONE_DIR" ls-remote --exit-code --tags origin "refs/tags/${TAG}" >/dev/null 2>&1; then
    echo "ERROR: tag ${TAG} already exists in gitolite -- ${VERSION} is already released." >&2
    exit 1
fi

# --- Step 2: stage ebuild + generate Manifest against the real distfile ---
echo "[2/6] Staging ebuild and generating Manifest..."
DEST="${CLONE_DIR}/${PKG_REL}"
mkdir -p "${DEST}/files"
cp "${EBUILD_DIR}/${EBUILD}" "${DEST}/"
[ -f "${EBUILD_DIR}/metadata.xml" ] && cp "${EBUILD_DIR}/metadata.xml" "${DEST}/"
if [ -d "${EBUILD_DIR}/files" ]; then
    for f in "${EBUILD_DIR}"/files/*; do [ -f "$f" ] && cp "$f" "${DEST}/files/"; done
fi
# Manifest checksums need the distfile locally under the SRC_URI name. Run the
# non-root ebuild in a private PORTAGE_TMPDIR so it needs no system dirs.
WORKTMP="$(mktemp -d)"
mkdir -p "${WORKTMP}/distdir"
cp "$TARBALL" "${WORKTMP}/distdir/${TARBALL_BN}"
rm -f "${DEST}/Manifest"
PORTAGE_TMPDIR="$WORKTMP" DISTDIR="${WORKTMP}/distdir" ebuild "${DEST}/${EBUILD}" manifest
rm -rf "$WORKTMP"

# --- Step 3: commit + tag in the source of truth ---
echo "[3/6] Committing and tagging ${TAG}..."
git -C "$CLONE_DIR" add "${PKG_REL}/"
git -C "$CLONE_DIR" "${GIT_ID[@]}" commit -m "${PKG_REL}: release ${VERSION}"
git -C "$CLONE_DIR" "${GIT_ID[@]}" tag -a "$TAG" -m "${PACKAGE} ${VERSION}"

# --- Step 4: push commit + tag to gitolite (atomic: both land or neither) ---
echo "[4/6] Pushing to gitolite..."
git -C "$CLONE_DIR" push --atomic origin master "refs/tags/${TAG}"

# --- Step 5: publish the distfile (deferred to here) ---
# Uploading only after the gitolite push means an abort before this point leaves
# NO orphan tarball in FILES_DIR -- nothing lands unless the release commits.
echo "[5/6] Uploading tarball..."
scp "$TARBALL" "${REMOTE_HOST}:/tmp/"
ssh "$REMOTE_HOST" "sudo mv /tmp/${TARBALL_BN} ${FILES_DIR}/ && sudo chown apache:apache ${FILES_DIR}/${TARBALL_BN}"

# --- Step 6: promote the distribution bare to this release tag ---
echo "[6/6] Promoting distribution bare to ${TAG}..."
"${SCRIPT_DIR}/promote-to-bare.sh" "$TAG"

echo ""
echo "=== Released ${PKG_REL}-${VERSION} ==="
echo "Clients pick it up on next sync:  sudo emaint sync -r spectraq && emerge -av ${PKG_REL}"
