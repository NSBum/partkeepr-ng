#!/bin/bash
# Promote the HTTPS distribution overlay (the bare repo that production hosts
# emerge from) to a specific gitolite release tag.
#
# This is the ONLY operation that changes what clients see. It is deliberate,
# manual, and version-keyed by design: gitolite (the source of truth) may take
# commits freely, but none reach hosts until a tag is promoted here. Reusable on
# its own for any package whose release commit + tag already live in gitolite
# (e.g. a third-party ebuild added by hand, then promoted).
#
# Usage: promote-to-bare.sh <tag> [--force]
#   <tag>     release tag present in gitolite (e.g. qucs-s-26.1.0)
#   --force   allow a non-fast-forward promote (rollback / rewrite) -- unsafe
set -euo pipefail

TAG="${1:-}"
FORCE=0
[ "${2:-}" = "--force" ] && FORCE=1
if [ -z "$TAG" ]; then
    echo "Usage: $0 <tag> [--force]" >&2
    exit 1
fi

REMOTE_HOST="chrisbuck"
# Source of truth and distribution mirror live on the same host, so the promote
# is a root-side fetch from one bare into the other -- no gitolite-shell needed
# for a read, and only the distribution bare is written.
GITOLITE_BARE="/home/repos/repositories/spectraq-overlay.git"
HTTPS_BARE="/var/www/com.spectraq.sqgit/htdocs/spectraq.git"

echo "=== Promote distribution bare -> ${TAG} ==="

ssh "$REMOTE_HOST" "sudo sh -s" <<REMOTE
set -eu
TAG='${TAG}'
FORCE='${FORCE}'
GITOLITE_BARE='${GITOLITE_BARE}'
HTTPS_BARE='${HTTPS_BARE}'

# A tag can only be promoted once it exists in the source of truth.
if ! git -C "\$GITOLITE_BARE" rev-parse -q --verify "refs/tags/\$TAG^{commit}" >/dev/null; then
    echo "ERROR: tag \$TAG not found in gitolite (\$GITOLITE_BARE)" >&2
    exit 1
fi

# Pull the tag (and the objects it needs) into the distribution bare.
git -C "\$HTTPS_BARE" fetch --no-tags "\$GITOLITE_BARE" "refs/tags/\$TAG:refs/tags/\$TAG"

NEW=\$(git -C "\$HTTPS_BARE" rev-list -n1 "refs/tags/\$TAG")
OLD=\$(git -C "\$HTTPS_BARE" rev-parse -q --verify refs/heads/master || echo "")

if [ "\$NEW" = "\$OLD" ]; then
    echo "Already at \$TAG (\$NEW); nothing to promote."
    exit 0
fi

# Production hosts sync this ref. Refuse anything but a fast-forward unless
# forced, so a release can never silently roll clients back or ship a rewrite.
if [ -n "\$OLD" ] && [ "\$FORCE" != "1" ]; then
    if ! git -C "\$HTTPS_BARE" merge-base --is-ancestor "\$OLD" "\$NEW"; then
        echo "ERROR: \$TAG (\$NEW) is not a fast-forward from current master (\$OLD)." >&2
        echo "       Re-run with --force only if a rollback/rewrite is intended." >&2
        exit 1
    fi
fi

git -C "\$HTTPS_BARE" update-ref refs/heads/master "\$NEW"
# Dumb-HTTP transport serves cached refs from info/refs; without this, clients
# fetch stale refs and never see the new release.
git -C "\$HTTPS_BARE" update-server-info

echo "promoted master: \${OLD:-(none)} -> \$NEW  (tag \$TAG)"
REMOTE

echo "=== Done. Clients get it on next sync (emaint sync -r spectraq). ==="
