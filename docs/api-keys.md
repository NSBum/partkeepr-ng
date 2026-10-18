# API keys

Browser sessions and API keys are independent authentication methods. Sign in,
open the username menu, and choose **API keys** to manage credentials for scripts
and integrations. Existing cookie-based API clients continue to work.

## Create and manage keys

Give each integration its own descriptive key name. Choose:

- **Read-only** (the default): permits GET and HEAD requests.
- **Read / write**: also permits inventory-changing requests as the assigned user.
- An expiry of 30 days, 90 days (default), one year, or Never.

The full key is displayed only in the creation window. Copy it before closing;
it cannot be retrieved later. Only a SHA-256 hash and an identifying prefix are
stored in the database. Closing and reopening the manager lets you create another
key. No key is saved to browser local storage.

The **Copy** button supports HTTPS clipboard access and falls back to copying
the selected text on HTTP installations. If the browser blocks both methods,
the key stays selected for Ctrl+C, Command+C, or the input's Copy context-menu
action. If you closed the window without saving the key, revoke it and create
a replacement; the existing key cannot be recovered from its hash.

The manager lists names, prefixes, owners (for administrators), permissions,
creation and expiry times, last-used times, and active/expired/revoked status.
Select a row to rename or revoke it. Revocation is permanent; create a new key
when replacing one. Last-used times are updated at most once per minute.

Users manage their own keys. Administrators can list, rename, and revoke all
keys, and assign newly created keys to another active account. Assignment gives
the key that account's identity, not the administrator's. Keys are not transferable
after creation; revoke and recreate when changing an integration's account or
permissions. At most 100 unexpired, non-revoked keys are allowed per account.

## Use a key

Send the key in an HTTP Authorization header:

```bash
curl -H "Authorization: Bearer $PARTKEEPR_API_KEY" \
    http://10.0.0.1:8082/api/parts
```

Set `PARTKEEPR_API_KEY` in your integration's secret configuration. Use HTTPS or
the private WireGuard connection for access. Keys are not accepted in URL query
parameters. An explicitly supplied invalid Authorization header returns 401 even
if the request also contains a valid browser session cookie.

API keys work on protected `/api/` and `/files/` endpoints, including
`GET /api/me` to identify the assigned user. Read-only keys reject write methods
with 403. Both key types reject account and key-management routes under
`/api/me/` with 403: use a password-authenticated session for those operations.
Keys never issue browser session cookies or create replacement keys.

Expired/revoked keys, and keys assigned to disabled, locked, or inactive accounts,
return 401. A key's effective account identity and administrator flag are resolved
on each request. Inventory changes are attributed to its assigned user.
Password changes invalidate browser sessions but do not revoke integration keys;
revoke unwanted keys separately. An already-running request may finish when its
key is revoked; subsequent requests are rejected.

## Management API (browser session required)

| Method | Endpoint | Purpose |
|---|---|---|
| GET | `/api/me/api-keys` | List your keys, or all keys for administrators |
| POST | `/api/me/api-keys` | Create a key; returns the secret once with `Cache-Control: no-store` |
| PATCH | `/api/me/api-keys/{id}` | Rename a key using `{"name":"New name"}` |
| DELETE | `/api/me/api-keys/{id}` | Revoke a key; repeated revocation is harmless |
| GET | `/api/me/api-key-owners` | List assignable active accounts (yourself for non-administrators) |

Example creation body:

```json
{
  "name": "Stock reporting",
  "access": "read",
  "expires_in_days": 90
}
```

`access` is `read` or `read_write`. `expires_in_days` is 1–3650, or null/omitted
for no expiry. `user_id` can be supplied by an administrator to assign the key;
otherwise it defaults to the signed-in account. Metadata endpoints never return
a full key or its stored hash. Attempts to manage someone else's key return 404
for non-administrators.

## Database upgrade

Apply `scripts/migrations/0007-api-keys.sql` before running the updated backend.
It adds the separate `ApiKey` table and can be reapplied safely. On an existing
installation, apply this file only; do not rerun the original migrations.
For new installations, the migration wrapper includes it in filename order.
Back up the `ApiKey` table with the rest of the database. Restoring an older
backup also restores its key/revocation state, so review keys after a restore.


## Verification

Run the Rust unit tests with `cargo test --bin partkeepr-api`. The integration
script creates disposable regular/admin users, keys, and one project, then
removes them. Run it against a test installation using a MariaDB account with
fixture-creation privileges:

```bash
python3 scripts/test-api-keys.py --base-url http://127.0.0.1:18082 \
    --mariadb /path/to/mariadb --socket /path/to/mysql.sock \
    --database partkeepr_test
```

The script uses local Unix-socket database authentication and never prints key
secrets or HTTP response bodies in assertion errors. The browser workflow is
covered in [manual smoke tests](manual-smoke-tests.md#api-keys).
