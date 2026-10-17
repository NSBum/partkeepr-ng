# Changelog

Release versions follow MAJOR.MINOR.PATCH. While the application is pre-1.0,
feature releases increment MINOR and fixes alone increment PATCH. Git commits
between releases do not each require a version bump. See
[versioning and release notes](docs/versioning.md) for the workflow.

## [0.2.0] - 2026-10-17

### Added

- Username account menu with My profile, Change password, Preferences, API keys,
  and Sign out. Profile email is editable; username and role are read-only.
- Per-account preferences for the starting section and maximum displayed parts,
  persisted across browsers.
- API-key authentication alongside existing session cookies. Keys can be named,
  assigned by administrators, restricted to read-only or read/write access,
  given an expiry, renamed, and revoked. The manager shows prefixes, ownership,
  status, and last-use timestamps; full secrets appear only once.
- Native systemd user-service templates, isolated native MariaDB deployment
  guidance, and a Docker-free backup workflow.
- Version helper that updates both Cargo manifests and Cargo.lock together, plus
  a consistency/changelog check in the release preflight.
- Repeatable API-key integration tests and account/menu smoke-test guidance.

### Fixed

- Sign out disappearing outside narrower browser windows. The workspace now
  scrolls independently while header controls remain visible.
- Original PartKeepr password compatibility: accept its single-iteration SHA-512
  hashes as well as the 5000-iteration variant; handle empty salts consistently.
- Migration documentation now maps the original separate file/image directories
  into the backend's entity directories and describes partial migration failures
  and rollback accurately.

### Security

- Password changes require the current password, confirmation, and a new password
  of at least 12 characters. Salt rotation invalidates older browser sessions.
- API keys are stored only as SHA-256 hashes, checked against current account
  status on every request, and rejected immediately on subsequent requests after
  revocation or expiry. Invalid bearer credentials never fall back to cookies.
- Account settings and key management require a password-authenticated browser
  session; API keys cannot create replacement keys or change account credentials.

### Upgrade

- Apply **only** `scripts/migrations/0007-api-keys.sql` to an installation that
  already has migrations 0001–0006. It adds the separate `ApiKey` table.
- Existing password hashes, sessions created by the account-menu version, and
  inventory remain compatible. Sessions predating the account-menu version
  require a fresh sign-in. Password changes do not revoke integration keys;
  revoke those separately when needed.
- Hard-refresh the browser after updating the backend and frontend together.

## [0.1.13]

Previous compiled-version baseline. Earlier releases did not maintain this
changelog; historical release notes have not been reconstructed.
