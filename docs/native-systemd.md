# Native deployment with systemd user services

Docker is optional. The Rust executable and static frontend can run directly
on Linux, alongside native MariaDB 10.6 or newer. The templates in
`contrib/systemd/` describe the separate MariaDB 10.11 deployment used during
migration, including a native backup script. No application secrets, database
contents, uploads, downloaded binaries, or local backups belong in Git.

These instructions assume Debian 12, an existing stable Rust toolchain, and a
checkout at `~/partkeepr-ng`. Both services run under your login account. For a
different checkout, edit the `%h/partkeepr-ng` paths in the service templates.

## Prepare the native binaries

If the host already runs MySQL, installing Debian's `mariadb-server` package
can remove conflicting MySQL packages. Check `apt-get -s install mariadb-server`
before making changes. The private installation below leaves existing MySQL
packages and data untouched and runs MariaDB on `127.0.0.1:3307`.

From the checkout root:

```bash
cargo +stable build --locked --release --bin partkeepr-api
mkdir -p local-install/native/{bin,packages,mariadb,database,backups}
mkdir -p local-install/storage
chmod 700 local-install local-install/native/database local-install/native/backups
cp target/release/partkeepr-api local-install/native/bin/

(cd local-install/native/packages && apt-get download \
    mariadb-server-core mariadb-server mariadb-client-core mariadb-client \
    mariadb-common liburing2)
for package in local-install/native/packages/*.deb; do
    dpkg-deb -x "$package" local-install/native/mariadb
done
```

Use matching MariaDB 10.11 packages from Debian's configured authenticated
repositories. These commands extract packages without installing them or running
package installation scripts. Check the private `mariadbd` executable with `ldd`
for other missing libraries on your host; `liburing2` is included because it was
absent on the migration host. Private binaries are **not updated by apt upgrade**.

## Initialize MariaDB and configure the app

Initialize only a new, empty data directory:

```bash
native_root="$PWD/local-install/native"
export LD_LIBRARY_PATH="$native_root/mariadb/usr/lib/x86_64-linux-gnu"
"$native_root/mariadb/usr/bin/mariadb-install-db" --no-defaults \
    --basedir="$native_root/mariadb/usr" --datadir="$native_root/database" \
    --auth-root-authentication-method=socket \
    --auth-root-socket-user="$(id -un)" --skip-test-db
```

Render the configuration templates with the checkout and runtime paths:

```bash
python3 - <<'PY'
from pathlib import Path
import os
checkout = Path.cwd().resolve()
runtime = os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}')
for name in ['app.env', 'mariadb.cnf']:
    text = (checkout / 'contrib/systemd' / (name + '.example')).read_text()
    text = text.replace('@CHECKOUT@', str(checkout)).replace('@RUNTIME_DIR@', runtime)
    target = checkout / 'local-install/native' / name
    if target.exists():
        raise SystemExit(f'Refusing to overwrite {target}')
    target.write_text(text)
    target.chmod(0o600)
PY
```

Edit `local-install/native/app.env`: set the database password and a random
session secret (`openssl rand -hex 48`). See `backend/.env.example` for optional
integrations. Set the storage path to the imported entity-folder tree.
The default bind address exposes port 8082 on IPv4 interfaces, including LAN and
WireGuard. Choose `127.0.0.1:8082` for local access only.

Install the user units and start the database:

```bash
mkdir -p ~/.config/systemd/user
cp contrib/systemd/*.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user start partkeepr-ng-db

local-install/native/mariadb/usr/bin/mariadb --no-defaults \
    --socket="$XDG_RUNTIME_DIR/partkeepr-ng-db/mysql.sock" -u "$(id -un)"
```

In the MariaDB prompt, create the application database and user, using the same
password as `app.env`:

```sql
CREATE DATABASE partkeepr_ng CHARACTER SET utf8mb3 COLLATE utf8mb3_unicode_ci;
CREATE USER 'partkeepr_ng'@'127.0.0.1' IDENTIFIED BY 'REPLACE_WITH_DATABASE_PASSWORD';
GRANT ALL ON partkeepr_ng.* TO 'partkeepr_ng'@'127.0.0.1';
```

Restore your database with the private client and socket above. For an original
PartKeepr dump, follow [the migration guide](migrating-from-partkeepr-1.4.0.md),
applying migrations to this instance on port 3307. For a dump from an already
migrated partkeepr-ng database, **do not repeat the migrations**.

## Cut over and verify

Stop the previous app before the final database dump and keep it stopped while
copying files and importing the dump. Keep backups of both. Compare table counts
and, where supported, `CHECKSUM TABLE ... EXTENDED` before switching services.
Stop anything else listening on port 8082, then:

```bash
systemctl --user enable --now partkeepr-ng-db partkeepr-ng
loginctl enable-linger "$USER"
loginctl show-user "$USER" -p Linger
```

Lingering keeps the user manager running after logout and starts enabled user
services at boot. Enabling it may require administrator authorization on some
hosts. Verify `http://localhost:8082/health`, existing login credentials, stock,
file downloads, and thumbnails, plus the server's VPN/LAN URL if used remotely.

After verification, stop the superseded containers and disable their restart
policies. Keep their volumes for rollback. Do not change unrelated containers.
The original database copy becomes stale once users write to the native service.

## Operations and backups

```bash
systemctl --user status partkeepr-ng partkeepr-ng-db
journalctl --user -u partkeepr-ng -n 100
systemctl --user restart partkeepr-ng
./scripts/backup-native.sh
```

The backup script briefly stops the app, saves the database, files, and private
configuration under `local-install/native/backups/`, and restarts the app if it
was running. Only directories with a `COMPLETE` marker contain finished backups.
Copy backups to a separate machine or your regular backup destination.
Set `PARTKEEPR_NATIVE_ROOT` and `PARTKEEPR_STORAGE_DIR` when using different paths.
The files archive contains the storage directory's contents; extract it into the
chosen storage directory when restoring. Database access uses local Unix-socket
authentication as the service owner, with no password in command arguments.

To restart MariaDB, stop the app first, restart `partkeepr-ng-db`, then start the
app again. After a source update, rebuild and stop the app before replacing its
binary; review any new migration requirements before restarting.

For private MariaDB security updates, download matching current Debian 10.11
packages into a separate staging directory and check their library dependencies.
Back up, stop both services, replace the private binary tree, and start MariaDB
followed by the app. Follow MariaDB's upgrade instructions for version changes.
Keep the checkout and `local-install/` in place: the service paths and persistent
data reside there.
