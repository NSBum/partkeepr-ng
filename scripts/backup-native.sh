#!/bin/bash
# Back up the native systemd user deployment. Run as its owning user.
# Override PARTKEEPR_NATIVE_ROOT for a checkout outside ~/partkeepr-ng.
set -euo pipefail
umask 077
native_root="${PARTKEEPR_NATIVE_ROOT:-$HOME/partkeepr-ng/local-install/native}"
storage_dir="${PARTKEEPR_STORAGE_DIR:-$native_root/../storage}"
runtime_dir="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
db_user="$(id -un)"

# Check prerequisites before interrupting the app.
for file in "$native_root/mariadb/usr/bin/mariadb-dump" "$native_root/app.env" "$native_root/mariadb.cnf"; do
    if [[ ! -f "$file" ]]; then
        echo "Missing required file: $file" >&2
        exit 1
    fi
done
if [[ ! -d "$storage_dir" ]]; then
    echo "Storage directory not found: $storage_dir" >&2
    exit 1
fi
mkdir -p "$native_root/backups"
backup_dir="$(mktemp -d "$native_root/backups/$(date +%Y%m%d-%H%M%S)-XXXXXX")"
was_running=0
finish() {
    if [[ "$was_running" == 1 ]]; then
        systemctl --user start partkeepr-ng.service
    fi
}
trap finish EXIT
if systemctl --user is-active --quiet partkeepr-ng.service; then
    was_running=1
    systemctl --user stop partkeepr-ng.service
fi
"$native_root/mariadb/usr/bin/mariadb-dump" --no-defaults \
    --socket="$runtime_dir/partkeepr-ng-db/mysql.sock" -u "$db_user" \
    --single-transaction --routines --triggers partkeepr_ng > "$backup_dir/database.sql"
tar -czf "$backup_dir/files.tar.gz" -C "$storage_dir" .
cp "$native_root/app.env" "$native_root/mariadb.cnf" "$backup_dir/"
touch "$backup_dir/COMPLETE"
printf 'Backup saved: %s\n' "$backup_dir"
