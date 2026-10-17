# Versioning and releases

The project uses numeric semantic versions (`MAJOR.MINOR.PATCH`). The version
compiled into the Rust executable is returned by `/api/version` and displayed in
the application header. Frontend code reads that value rather than maintaining
its own version.

While the application is pre-1.0:

- Increment **MINOR** for a feature release, or for incompatible changes. Describe
  any migration or compatibility requirements explicitly in the changelog.
- Increment **PATCH** for releases containing only backward-compatible fixes.
- Group related commits into a release; do not bump for every commit or count
  features as separate releases. Version 0.2.0 groups the account menu, API keys,
  and the missing Sign out fix since the 0.1.13 baseline.
- Reserve **1.0.0** for an explicit stable-release decision, not a routine feature.

## Update and check the version

Python 3.11 or newer is needed for this development helper:

```bash
python3 scripts/version.py                 # show the current version
python3 scripts/version.py --bump patch    # e.g. 0.2.0 -> 0.2.1
python3 scripts/version.py --bump minor    # e.g. 0.2.0 -> 0.3.0
python3 scripts/version.py --set 0.3.0      # choose an explicit higher version
```

These update `Cargo.toml`, `backend/Cargo.toml`, and the `partkeepr-api` entry in
`Cargo.lock`. Two manifest declarations are intentional: release tooling reads
the workspace version, while the distribution can ship `backend/` standalone.
The helper refuses inconsistent starting versions or a downgrade/reused version.

Add a `## [VERSION] - YYYY-MM-DD` entry to `CHANGELOG.md`, covering user-visible
behavior, fixes, security changes, and upgrade steps. Do not invent intermediate
versions for already-landed commits. Then validate:

```bash
python3 scripts/version.py --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test-version.py
cargo +stable test --locked --release --bin partkeepr-api
cargo +stable build --locked --release --bin partkeepr-api
```

Review relevant integration/smoke tests, then commit the version, changelog, and
release changes together. A local commit does not publish or tag a release.

## Existing publication tooling

`scripts/release.sh` performs preflight checks, builds a distribution, pushes the
source repository, creates a `vVERSION` tag, and publishes to the Gentoo overlay.
Use it only when intentionally publishing. Its `--check` option performs checks
without publishing, including the version/changelog check.

The overlay workflow also requires a matching `ebuild/partkeepr-ng-VERSION.ebuild`
and companion files. This checkout does not include the ebuild directory, so
that publication path requires packaging to be supplied separately. Native
systemd deployments can build/install the versioned executable without it.

To update the native installation, follow [native deployment](native-systemd.md):
apply any new migrations, stop the app, replace the executable and frontend, and
restart. Verify `/api/version` and the header report the expected version.
