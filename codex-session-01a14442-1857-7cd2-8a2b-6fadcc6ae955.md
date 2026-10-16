# Codex conversation

## User

Please advise me on how to install. I would also like to bring in my existing database, attachments and images, etc. from the original PartKeepr in these ways:

My existing database dump from PartKeepr (OG) is at /tmp/partkeepr-prod.sql
and I dumped the PartKeepr files in this way:

```
docker exec partkeepr-partkeepr-1 ls -lah /app/data
total 24K
drwxrwxrwx    5 www-data www-data    4.0K Jan 17  2026 .
drwxr-xr-x    1 www-data www-data    4.0K Jun 17  2022 ..
drwxr-xr-x    5 www-data www-data    4.0K Jan 17  2026 files
drwxr-xr-x    5 www-data www-data    4.0K Jan 17  2026 images
drwxr-xr-x    2 www-data www-data    4.0K Feb 11  2026 temp
➜  ~ docker exec partkeepr-partkeepr-1 \
    tar -czf - -C /app data \
    > ~/partkeepr-files.tar.gz
```

## Assistant

I’ll check the installation and migration support in this repository, then outline how to import your database and data archive.

## Activity

    $ pwd && rg --files -g 'AGENTS.md' -g '*README*' -g '*INSTALL*' -g '*migrat*' -g '*compose*' -g '*.env*' -g 'pyproject.toml' -g 'package.json'
    /home/alan/partkeepr-ng
    backend/.env.example
    backend/README.md
    docs/live-schema/partkeepr-migrations.txt
    README.md
    docs/upstream-issues/README.md
    docs/migrating-from-partkeepr-1.4.0.md
    scripts/migrate-from-upstream.sh
    scripts/jlcparts-build/README.md
    scripts/jlcparts-build/jlcparts/migrate.py
    frontend/vendor/webix/legacy/excanvas/README.txt
    ✓ • 0ms

## Activity

    $ cat Cargo.toml backend/Cargo.toml scripts/migrations/*.sql; rg -n 'storage_dir|STORAGE_DIR|images/|files/|dotenv|FRONTEND|health' backend/src/main.rs backend/src/services backend/src/handlers/attachments.rs; ls -lh /tmp/partkeepr-prod.sql /home/alan/partkeepr-files.tar.gz; rg -n '^(CREATE DATABASE|USE |-- Server version|-- Current Database|CREATE TABLE)' /tmp/partkeepr-prod.sql
    [workspace]
    members = ["backend"]
    resolver = "2"
    
    # Workspace-level package metadata. The driving reason for declaring a
    # version here is the SpectraQ overlay release driver — it validates
    # the tagged version against `grep '^version' Cargo.toml`, which fails
    # silently under `set -euo pipefail` if the file has no `version =`
    # line. Note: `backend/Cargo.toml` does NOT inherit via
    # `version.workspace = true` — it keeps its own static `version = "X.Y.Z"`
    # because `scripts/make-dist.sh` ships `backend/` standalone (no
    # workspace root in the tarball), so the inherited form can't resolve
    # at build time on chrisbuck. Both files must be bumped together at
    # release time.
    [workspace.package]
    version = "0.1.13"
    [package]
    name = "partkeepr-api"
    # Two version declarations — this one (consumed by cargo build) and
    # the top-level Cargo.toml's [workspace.package] version (consumed by
    # spectraq-release-driver.sh on chrisbuck via `grep ^version`). Both
    # must be bumped together at release time. We use static versions on
    # both ends because cargo workspace-inheritance breaks the tarball
    # pipeline — the driver's make-dist.sh ships backend/ standalone,
    # without the workspace root, so `version.workspace = true` can't
    # resolve at build time on chrisbuck.
    version = "0.1.13"
    edition = "2021"
    publish = false
    authors = ["Capt Frito <capt.frito@gmail.com>"]
    license = "GPL-3.0-or-later"
    description = "Backend for partkeepr-ng — a Rust+Webix rewrite of PartKeepr 1.4.0 electronic-parts inventory"
    repository = "https://github.com/CaptFrito/partkeepr-ng"
    readme = "../README.md"
    # `cargo run` (no --bin) starts the API server. The reset-password
    # binary is for one-off CLI use via `cargo run --bin reset-password`.
    default-run = "partkeepr-api"
    
    [dependencies]
    anyhow = "1"
    axum = { version = "0.7", features = ["multipart"] }
    bytes = "1"
    # Streaming body support for the URL-fetch path (cap by reading chunks).
    futures-util = { version = "0.3", default-features = false }
    chrono = { version = "0.4", default-features = false, features = ["serde", "clock"] }
    dotenvy = "0.15"
    # Slice 7: image thumbnail generation. Default features pull in heavy
    # format support (TIFF, PNM, ICO, etc.) we don't need; restrict to the
    # common formats we actually expect to handle.
    image = { version = "0.25", default-features = false, features = ["jpeg", "png", "gif", "webp"] }
    mime = "0.3"
    mime_guess = "2"
    # Slice 7 follow-up: server-side URL fetching for "Attach by URL".
    # rustls-tls keeps the dep tree native-Rust (no OpenSSL), stream
    # enables streaming response bodies (we cap upload size while reading).
    reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream", "json"] }
    rust_decimal = { version = "1", features = ["serde-with-str"] }
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    sqlx = { version = "0.8", default-features = false, features = [
        "runtime-tokio-rustls",
        "mysql",
        "sqlite",      # read-only access to the jlcparts cache.sqlite3
        "chrono",
        "rust_decimal",
        "macros",
    ] }
    tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "net", "fs", "io-util", "process"] }
    tower-http = { version = "0.5", features = ["trace", "cors", "fs"] }
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["env-filter"] }
    uuid = { version = "1", features = ["v4"] }
    # Slice 10: auth — FOSUserBundle password verification (sha512 + 5000 iter
    # + base64) and HMAC-signed session-cookie payloads.
    sha2 = "0.10"
    hmac = "0.12"
    base64 = "0.22"
    subtle = "2.6"
    tower = "0.5"
    # axum-extra gets us cookie-jar extractors so handlers can read/write
    # cookies without manually stitching headers.
    axum-extra = { version = "0.9", features = ["cookie"] }
    # Used for cookie max-age durations (the cookie crate's API takes
    # `time::Duration`, not `std::time::Duration` or `chrono::Duration`).
    time = "0.3"
    -- Slice 10c-1: per-user attribution backfill
    --
    -- Adds a "system" PartKeeprUser row, then assigns it to every Project
    -- and StockEntry that currently has user_id IS NULL (everything written
    -- by partkeepr-ng before slice 10c, plus any legacy-import gaps).
    --
    -- Idempotent: safe to re-run. Uses INSERT ... WHERE NOT EXISTS for the
    -- system row and UPDATE ... WHERE user_id IS NULL for the backfill.
    --
    -- Run against partkeepr_dev (cutover: against prod copy, post import,
    -- pre service start).
    
    INSERT INTO PartKeeprUser (username, password, admin, lastSeen, provider_id,
                               email, legacy, active, protected)
      SELECT 'system', NULL, 0, NULL, 1, NULL, 0, 0, 1
      FROM DUAL
      WHERE NOT EXISTS (SELECT 1 FROM PartKeeprUser WHERE username = 'system');
    
    SET @sysid := (SELECT id FROM PartKeeprUser WHERE username = 'system');
    
    UPDATE StockEntry SET user_id = @sysid WHERE user_id IS NULL;
    UPDATE Project    SET user_id = @sysid WHERE user_id IS NULL;
    
    SELECT @sysid AS system_user_id,
           (SELECT COUNT(*) FROM StockEntry WHERE user_id IS NULL) AS stock_null_after,
           (SELECT COUNT(*) FROM Project    WHERE user_id IS NULL) AS project_null_after;
    -- Multi-location parts (PartStorageLocation join table).
    --
    -- Adds a per-part breakdown of where stock physically lives — loose
    -- in a bin, on a reel, mounted to a P&P feeder, etc. Existing
    -- `Part.storageLocation_id` and `Part.stockLevel` continue to be the
    -- "primary location" + "total on hand"; this table is descriptive,
    -- kept in sync by the operator. See plan: docs of W7c.X for the
    -- rationale (descriptive-only v1).
    --
    -- Idempotent: re-runnable without effect if the table already exists.
    -- Apply with:
    --   mariadb -u partkeepr_dev -p... partkeepr_dev \
    --     < scripts/migrations/multi-location-parts.sql
    
    CREATE TABLE IF NOT EXISTS PartStorageLocation (
        id                  INT(11) NOT NULL AUTO_INCREMENT,
        part_id             INT(11) NOT NULL,
        storageLocation_id  INT(11) NOT NULL,
        form                ENUM('Loose','Reel','CutTape','Tray','Tube',
                                 'Feeder','Bag','Other')
                            NOT NULL DEFAULT 'Loose',
        quantity            INT(11) NOT NULL DEFAULT 0,
        lotNumber           VARCHAR(255) DEFAULT NULL,
        comment             TEXT DEFAULT NULL,
        created             DATETIME DEFAULT NULL,
        lastModified        DATETIME DEFAULT NULL,
        user_id             INT(11) DEFAULT NULL,
        PRIMARY KEY (id),
        KEY IDX_PSL_part (part_id),
        KEY IDX_PSL_loc  (storageLocation_id),
        CONSTRAINT FK_PSL_part FOREIGN KEY (part_id)
          REFERENCES Part (id) ON DELETE CASCADE,
        CONSTRAINT FK_PSL_loc  FOREIGN KEY (storageLocation_id)
          REFERENCES StorageLocation (id),
        CONSTRAINT FK_PSL_user FOREIGN KEY (user_id)
          REFERENCES PartKeeprUser (id) ON DELETE SET NULL
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
    -- Slice 12b.2 follow-on: structured order tracking on StockEntry.
    --
    -- Adds two columns so we can answer "which sales orders contributed
    -- to this part's stock?" without parsing the freeform `comment`
    -- field. Both NULL for legacy rows + manual entries; populated by
    -- the Digi-Key Order Status receive flow (and Mouser Order History
    -- once that ships).
    --
    -- Backfills any rows the receive flow already wrote (slice 12b.2
    -- shipped 2026-05-08; the comment format is "Digi-Key SO #N line K").
    -- Idempotent: safe to re-run — uses IF NOT EXISTS / IF NOT NULL
    -- predicates throughout.
    --
    -- Apply with:
    --   mariadb -u partkeepr_dev -p... partkeepr_dev \
    --     < scripts/migrations/stock-entry-order-tracking.sql
    
    -- 1. Schema additions. MariaDB doesn't support IF NOT EXISTS on
    --    individual ADD COLUMN clauses (8.0.29+ does in MySQL but not
    --    MariaDB stable), so guard via information_schema lookups.
    
    SET @col_exists := (
        SELECT COUNT(*)
        FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'StockEntry'
          AND COLUMN_NAME = 'distributor_id'
    );
    SET @sql := IF(@col_exists = 0,
        'ALTER TABLE StockEntry ADD COLUMN distributor_id INT(11) NULL DEFAULT NULL',
        'SELECT "distributor_id already present" AS info');
    PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;
    
    SET @col_exists := (
        SELECT COUNT(*)
        FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'StockEntry'
          AND COLUMN_NAME = 'salesOrderNumber'
    );
    SET @sql := IF(@col_exists = 0,
        'ALTER TABLE StockEntry ADD COLUMN salesOrderNumber VARCHAR(64) NULL DEFAULT NULL',
        'SELECT "salesOrderNumber already present" AS info');
    PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;
    
    -- 2. Index on the lookup pair. Used by the per-part receipts
    --    aggregation endpoint and "find every part that came from this
    --    SO" admin queries.
    
    SET @idx_exists := (
        SELECT COUNT(*)
        FROM information_schema.STATISTICS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'StockEntry'
          AND INDEX_NAME = 'idx_se_dist_so'
    );
    SET @sql := IF(@idx_exists = 0,
        'CREATE INDEX idx_se_dist_so ON StockEntry (distributor_id, salesOrderNumber)',
        'SELECT "idx_se_dist_so already present" AS info');
    PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;
    
    -- 3. Foreign key — soft (no CASCADE; orphans get a NULL distributor_id
    --    if a Distributor is deleted, but we hard-block delete in the
    --    Distributor merge handler anyway).
    
    SET @fk_exists := (
        SELECT COUNT(*)
        FROM information_schema.TABLE_CONSTRAINTS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'StockEntry'
          AND CONSTRAINT_NAME = 'fk_StockEntry_distributor'
    );
    SET @sql := IF(@fk_exists = 0,
        'ALTER TABLE StockEntry ADD CONSTRAINT fk_StockEntry_distributor \
         FOREIGN KEY (distributor_id) REFERENCES Distributor(id) ON DELETE SET NULL',
        'SELECT "fk_StockEntry_distributor already present" AS info');
    PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;
    
    -- 4. Backfill: rows whose comment matches "Digi-Key SO #<N>..." get
    --    the structured fields populated. Match the SO# greedily up to
    --    the next non-digit. Only touch rows where distributor_id IS NULL
    --    so re-running this is a no-op.
    
    SET @dk_id := (SELECT id FROM Distributor WHERE LOWER(name) = 'digi-key' LIMIT 1);
    
    -- MariaDB's REGEXP_SUBSTR is 2-arg (str, pattern); we want the first
    -- run of digits after "SO #", which is also "the only digit run in
    -- the comment up to the trailing ' line K'". Split off the prefix
    -- before the digits, then take everything up to (but not including)
    -- the next space.
    UPDATE StockEntry
    SET distributor_id   = @dk_id,
        salesOrderNumber = SUBSTRING_INDEX(
                             SUBSTRING(comment, LENGTH('Digi-Key SO #') + 1),
                             ' ', 1)
    WHERE @dk_id IS NOT NULL
      AND distributor_id IS NULL
      AND comment LIKE 'Digi-Key SO #%';
    
    SELECT COUNT(*) AS backfilled_rows
    FROM StockEntry
    WHERE distributor_id = @dk_id
      AND salesOrderNumber IS NOT NULL;
    -- Slice 13b follow-on: auto-populate Container rows for parts that
    -- already have stock.
    --
    -- The Containers tab (PartStorageLocation rows) was added later than
    -- the rest of the schema, so existing parts with positive stockLevel
    -- have zero rows. Operator was forced to create a "Loose" container
    -- row covering the existing stock just to start using the feature.
    -- Backfill takes care of all of them in one shot.
    --
    -- Also: relax NOT NULL on storageLocation_id so a "loose, not yet
    -- binned" container is representable. The original schema required
    -- a bin be picked even for stock the operator hadn't yet placed.
    --
    -- Idempotent: re-runnable. Backfill skips parts that already have
    -- rows; nullability change is a no-op when applied twice.
    --
    -- Apply with:
    --   mariadb -u partkeepr_dev -p... partkeepr_dev \
    --     < scripts/migrations/container-rows-backfill.sql
    
    -- 1. Drop NOT NULL on storageLocation_id. MariaDB MODIFY COLUMN
    --    is idempotent (specifying NULL twice is harmless).
    
    ALTER TABLE PartStorageLocation
        MODIFY COLUMN storageLocation_id INT(11) NULL DEFAULT NULL;
    
    -- 2. Backfill: for every Part with stockLevel > 0 that has zero
    --    PartStorageLocation rows, insert one Loose row with the full
    --    quantity. storage_location_id = the part's primary
    --    Part.storageLocation_id (which is already a valid leaf in
    --    StorageLocation), or NULL if unset.
    --
    --    This preserves total stock balance: sum(PartStorageLocation.quantity)
    --    will equal Part.stockLevel for these parts immediately after.
    
    INSERT INTO PartStorageLocation
        (part_id, storageLocation_id, form, quantity, lotNumber, comment,
         user_id, created, lastModified)
    SELECT
        p.id,
        p.storageLocation_id,
        'Loose',
        p.stockLevel,
        NULL,
        'Auto-backfilled from existing stock',
        NULL,
        NOW(),
        NOW()
    FROM Part p
    LEFT JOIN PartStorageLocation psl ON psl.part_id = p.id
    WHERE p.stockLevel > 0
    GROUP BY p.id
    HAVING COUNT(psl.id) = 0;
    
    -- Report rows inserted (informational; the INSERT count is also
    -- visible in MariaDB's row-count output).
    
    SELECT COUNT(*) AS containers_after_backfill,
           SUM(quantity) AS total_qty_in_containers
    FROM PartStorageLocation;
    -- PartUnit cleanup: drop the vestigial Reel / Cut Tape rows.
    --
    -- The PartUnit table is meant to be the unit-of-measurement for
    -- `Part.stockLevel` ("how many of <unit> do you have?"). It got
    -- populated historically with reel/cut-tape sizes (8mm/12mm/16mm/
    -- 24mm × Reel/CutTape) when there was no other way to record
    -- container form on a part. PartStorageLocation.form now captures
    -- that per physical container (slice 13b), making the reel/cut-
    -- tape PartUnit rows redundant and confusing.
    --
    -- Adds proper non-electronic measurement units (Metres, Litres,
    -- Kilograms, etc.) so the lookup can serve the broader inventory
    -- (mechanical, automotive, tools, supplies).
    --
    -- Idempotent: skip rows already deleted, INSERT IGNORE on the
    -- new ones (UNIQUE on name).
    --
    -- Apply with:
    --   mariadb -u partkeepr_dev -p... partkeepr_dev \
    --     < scripts/migrations/partunit-cleanup.sql
    
    -- 1. Repoint any Part still using a reel/cut-tape PartUnit to
    --    Pieces (id 1). Container form is now in PartStorageLocation.
    
    UPDATE Part p
    JOIN PartUnit pu ON pu.id = p.partUnit_id
    SET p.partUnit_id = 1
    WHERE pu.name LIKE '%Parts Reel'
       OR pu.name LIKE '%Cut Tape';
    
    -- 2. Delete the eight vestigial rows. The UPDATE above guarantees
    --    no Part references them, so this won't FK-violate.
    
    DELETE FROM PartUnit
    WHERE name IN (
         '8mm Parts Reel',
        '12mm Parts Reel',
        '16mm Parts Reel',
        '24mm Parts Reel',
         '8mm Cut Tape',
        '12mm Cut Tape',
        '16mm Cut Tape',
        '24mm Cut Tape'
    );
    
    -- 3. Seed proper measurement units for non-electronic inventory.
    --    INSERT IGNORE relies on UNIQUE on name (or just no-ops on
    --    duplicate-key error if name isn't unique). PartUnit table in
    --    PartKeepr 1.4.0 has no UNIQUE on name so we guard via NOT EXISTS.
    
    INSERT INTO PartUnit (name, shortName, is_default)
    SELECT * FROM (
        SELECT 'Metres'      AS n, 'm'   AS s, 0 AS d UNION ALL
        SELECT 'Centimetres',     'cm',     0 UNION ALL
        SELECT 'Litres',          'L',      0 UNION ALL
        SELECT 'Millilitres',     'mL',     0 UNION ALL
        SELECT 'Kilograms',       'kg',     0 UNION ALL
        SELECT 'Grams',           'g',      0 UNION ALL
        SELECT 'Pairs',           'pr',     0 UNION ALL
        SELECT 'Sets',            'set',    0
    ) AS new_units
    WHERE NOT EXISTS (
        SELECT 1 FROM PartUnit pu WHERE pu.name = new_units.n
    );
    
    -- 4. Report final state.
    
    SELECT id, name, shortName, is_default
    FROM PartUnit
    ORDER BY is_default DESC, name;
    -- Slice 13c: tie each StockEntry to the specific PartStorageLocation
    -- row it added to / drew from. Nullable for back-compat: legacy entries
    -- (pre-2026-05-09) keep NULL, new entries from add/remove flows populate
    -- it. Combined with the auto-update of PartStorageLocation.quantity in
    -- the handler, this gives shop-floor accuracy for "which reel did this
    -- 50-piece draw come from?" without breaking the existing aggregate
    -- Part.stockLevel accounting.
    --
    -- Why nullable: existing 1036 StockEntry rows pre-date PartStorageLocation
    -- and there is no reliable way to retroactively assign them to specific
    -- containers. Leaving them NULL preserves history without inventing
    -- attributions that would be wrong half the time.
    --
    -- ON DELETE SET NULL: deleting a PartStorageLocation row should not
    -- cascade-delete the StockEntry history; the audit trail outlives the
    -- container. The FK is maintained so we can still join when present.
    
    ALTER TABLE StockEntry
      ADD COLUMN partStorageLocation_id INT(11) DEFAULT NULL AFTER salesOrderNumber;
    
    ALTER TABLE StockEntry
      ADD KEY idx_se_psl (partStorageLocation_id);
    
    ALTER TABLE StockEntry
      ADD CONSTRAINT fk_StockEntry_psl
        FOREIGN KEY (partStorageLocation_id)
        REFERENCES PartStorageLocation (id)
        ON DELETE SET NULL;
    rg: backend/src/services: No such file or directory (os error 2)
    backend/src/handlers/attachments.rs:15://! - **GET** `/files/:kind/:id` — download original (browser-friendly,
    backend/src/handlers/attachments.rs:17://! - **GET** `/files/:kind/:id/thumb` — JPEG thumbnail (image kinds
    backend/src/main.rs:65:    dotenvy::dotenv().ok();
    backend/src/main.rs:120:    // checkout layout). Production sets PARTKEEPR_FRONTEND_DIR to the
    backend/src/main.rs:122:    let frontend_dir: PathBuf = std::env::var("PARTKEEPR_FRONTEND_DIR")
    backend/src/main.rs:127:            "frontend directory {:?} does not exist (set PARTKEEPR_FRONTEND_DIR \
    backend/src/main.rs:135:        .route("/health", get(health))
    backend/src/main.rs:139:        // public paths (/health, /api/login) and any non-/api/, non-/files/
    backend/src/main.rs:159:async fn health() -> &'static str {
    -rw-r--r-- 1 alan alan 535M Oct 16 06:22 /home/alan/partkeepr-files.tar.gz
    -rw-r--r-- 1 alan alan 543K Oct 16 06:20 /tmp/partkeepr-prod.sql
    6:-- Server version    10.5.29-MariaDB-ubu2004
    26:CREATE TABLE `BatchJob` (
    51:CREATE TABLE `BatchJobQueryField` (
    81:CREATE TABLE `BatchJobUpdateField` (
    110:CREATE TABLE `CachedImage` (
    135:CREATE TABLE `CronLogger` (
    161:CREATE TABLE `Distributor` (
    194:CREATE TABLE `FOSUser` (
    234:CREATE TABLE `Footprint` (
    263:CREATE TABLE `FootprintAttachment` (
    297:CREATE TABLE `FootprintCategory` (
    332:CREATE TABLE `FootprintImage` (
    366:CREATE TABLE `GridPreset` (
    394:CREATE TABLE `ImportPreset` (
    420:CREATE TABLE `Manufacturer` (
    451:CREATE TABLE `ManufacturerICLogo` (
    485:CREATE TABLE `MetaPartParameterCriteria` (
    522:CREATE TABLE `Part` (
    572:CREATE TABLE `PartAttachment` (
    607:CREATE TABLE `PartCategory` (
    642:CREATE TABLE `PartDistributor` (
    677:CREATE TABLE `PartKeeprUser` (
    712:CREATE TABLE `PartManufacturer` (
    742:CREATE TABLE `PartParameter` (
    790:CREATE TABLE `PartUnit` (
    816:CREATE TABLE `Project` (
    843:CREATE TABLE `ProjectAttachment` (
    876:CREATE TABLE `ProjectPart` (
    909:CREATE TABLE `ProjectRun` (
    936:CREATE TABLE `ProjectRunPart` (
    966:CREATE TABLE `Report` (
    990:CREATE TABLE `ReportPart` (
    1022:CREATE TABLE `ReportProject` (
    1051:CREATE TABLE `SchemaVersions` (
    1074:CREATE TABLE `SiPrefix` (
    1101:CREATE TABLE `StatisticSnapshot` (
    1127:CREATE TABLE `StatisticSnapshotUnit` (
    1157:CREATE TABLE `StockEntry` (
    1191:CREATE TABLE `StorageLocation` (
    1219:CREATE TABLE `StorageLocationCategory` (
    1254:CREATE TABLE `StorageLocationImage` (
    1288:CREATE TABLE `SystemNotice` (
    1315:CREATE TABLE `SystemPreference` (
    1338:CREATE TABLE `TempImage` (
    1369:CREATE TABLE `TempUploadedFile` (
    1400:CREATE TABLE `TipOfTheDay` (
    1423:CREATE TABLE `TipOfTheDayHistory` (
    1449:CREATE TABLE `Unit` (
    1474:CREATE TABLE `UnitSiPrefixes` (
    1502:CREATE TABLE `UserPreference` (
    1529:CREATE TABLE `UserProvider` (
    ✓ • 0ms

## Activity

    $ rg -n 'STORAGE_DIR|join\(|iclogo|PartAttachment|images|files' backend/src/storage.rs; sed -n '1,220p' backend/src/storage.rs; tar -tzf /home/alan/partkeepr-files.tar.gz | head -35; command -v cargo rustc mariadb mysql docker; cat /etc/os-release
    rg: backend/src/storage.rs: IO error for operation on backend/src/storage.rs: No such file or directory (os error 2)
    sed: can't read backend/src/storage.rs: No such file or directory
    data/
    data/files/
    data/files/FootprintAttachment/
    data/files/FootprintAttachment/41a752cc-f385-11f0-9409-461e7faa8daf.pdf
    data/files/PartAttachment/
    data/files/PartAttachment/96099bf2-1231-11f1-8406-c3e2a24b13f5.pdf
    data/files/PartAttachment/bc290eba-f44c-11f0-ac95-8822a48f2b2d.pdf
    data/files/PartAttachment/4292a5f4-f88c-11f0-b040-64a50c026e91.pdf
    data/files/PartAttachment/9f2cd818-122f-11f1-958d-d0cda85b8fa5.pdf
    data/files/PartAttachment/4685f310-f3f9-11f0-9e3a-8dd9843b7c66.pdf
    data/files/PartAttachment/0bb847a8-f726-11f0-8158-e7278046478d.pdf
    data/files/PartAttachment/04685eb8-f891-11f0-b968-a3f0a33effa0.pdf
    data/files/PartAttachment/413556f4-f448-11f0-9dd1-3a86b5dcb193.pdf
    data/files/PartAttachment/3e28c594-f3ae-11f0-9ef9-e09faf29c7b0.pdf
    data/files/PartAttachment/27a05232-f40e-11f0-8418-3be469c1c3ff.pdf
    data/files/PartAttachment/fb2d1fe0-f4c2-11f0-be7f-6672c91261f7.pdf
    data/files/PartAttachment/edd64212-22a4-11f1-9502-a17340cda096.pdf
    data/files/PartAttachment/19583218-f395-11f0-9975-1a6649419220.pdf
    data/files/PartAttachment/87e312a8-2305-11f1-9f5c-a0a9b1bb644a.pdf
    data/files/PartAttachment/92d5ba6a-244c-11f1-9df9-1025743bc7fc.pdf
    data/files/PartAttachment/0ec4aa9a-f4c4-11f0-97de-ff0acf5d9624.pdf
    data/files/PartAttachment/ffe47fd4-f516-11f0-a779-71d662e07411.pdf
    data/files/PartAttachment/5f105736-f40d-11f0-9ae0-98d181a4503d.pdf
    data/files/PartAttachment/763b7cf0-f391-11f0-b936-cec3352b0cd8.pdf
    data/files/PartAttachment/89d3d11a-f3b2-11f0-8ef6-65c5b35f94fb.pdf
    data/files/PartAttachment/b10b4e5a-f3af-11f0-b0db-81f92b792464.pdf
    data/files/PartAttachment/885b5dc6-f4c5-11f0-af0a-5db24a4761a2.pdf
    data/files/PartAttachment/0fdd0134-f44c-11f0-9953-b0496895913b.pdf
    data/files/PartAttachment/462fdc60-f3f8-11f0-8c7b-1712f4b5c055.pdf
    data/files/PartAttachment/8587de2a-1230-11f1-8871-dffd5852c2da.pdf
    data/files/PartAttachment/14cf43dc-1170-11f1-b61b-e3afc1cb1afd.pdf
    data/files/PartAttachment/2caa71f2-f33e-11f0-81aa-42a0d98271ab.pdf
    data/files/PartAttachment/d81279dc-205b-11f1-aba3-5c23777491a2.pdf
    data/files/PartAttachment/3c6bb6e4-0dd0-11f1-b1bf-f1d7f29cfd70.pdf
    data/files/PartAttachment/34684f3c-f4c3-11f0-b9a8-2a703b3b707b.pdf
    tar: stdout: write error
    /home/alan/.cargo/bin/cargo
    /home/alan/.cargo/bin/rustc
    /usr/bin/mysql
    /usr/bin/docker
    PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
    NAME="Debian GNU/Linux"
    VERSION_ID="12"
    VERSION="12 (bookworm)"
    VERSION_CODENAME=bookworm
    ID=debian
    HOME_URL="https://www.debian.org/"
    SUPPORT_URL="https://www.debian.org/support"
    BUG_REPORT_URL="https://bugs.debian.org/"
    ✓ • 3.95s

## Activity

    $ rg -n 'STORAGE_DIR|iclogo|struct.*Storage|storage_root' backend/src; sed -n '1,180p' backend/src/files.rs
    backend/src/handlers/storage_locations.rs:33:pub struct StorageTreeNode {
    backend/src/handlers/storage_locations.rs:43:pub struct StorageLocationLite {
    backend/src/handlers/parts.rs:462:pub struct StorageLocationSummary {
    backend/src/handlers/storage.rs:9://! $PARTKEEPR_STORAGE_DIR/
    backend/src/handlers/storage.rs:34:pub struct StorageConfig {
    backend/src/handlers/storage.rs:40:    /// Read root from env (`PARTKEEPR_STORAGE_DIR`), falling back to
    backend/src/handlers/storage.rs:44:        let root = match env::var("PARTKEEPR_STORAGE_DIR") {
    backend/src/models/part_storage_location.rs:13:pub struct PartStorageLocation {
    backend/src/models/storage_location_image.rs:12:pub struct StorageLocationImage {
    backend/src/models/storage_location_category.rs:12:pub struct StorageLocationCategory {
    backend/src/models/storage_location.rs:12:pub struct StorageLocation {
    sed: can't read backend/src/files.rs: No such file or directory
    ✗ (2) • 0ms

## Activity

    $ cat backend/src/handlers/storage.rs; sed -n '1,145p' backend/src/handlers/attachments.rs; rustc --version; rg -n 'name = "(time|image)"|rust-version' Cargo.lock
    // SPDX-License-Identifier: GPL-3.0-or-later
    
    //! Slice 7: file-storage abstraction shared by all attachment endpoints.
    //!
    //! Layout on disk — matches the legacy PartKeepr layout so existing
    //! files can be `rsync`'d into the new tree without renaming:
    //!
    //! ```text
    //! $PARTKEEPR_STORAGE_DIR/
    //! ├── PartAttachment/{uuid}.{ext}
    //! ├── PartAttachment/{uuid}.thumb.jpg        (only for images)
    //! ├── FootprintImage/{uuid}.{ext}
    //! ├── FootprintAttachment/{uuid}.{ext}
    //! ├── ManufacturerICLogo/{uuid}.{ext}
    //! └── StorageLocationImage/{uuid}.{ext}
    //! ```
    //!
    //! Subdir name is the Doctrine entity class name. UUID filenames are
    //! preserved from the existing DB rows (`filename` column).
    
    use std::env;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    
    use anyhow::Context;
    use bytes::Bytes;
    use image::ImageReader;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;
    
    use super::clamd::{self, ClamdConfig, ScanResult};
    
    #[derive(Debug, Clone)]
    pub struct StorageConfig {
        pub root: PathBuf,
        pub clamd: ClamdConfig,
    }
    
    impl StorageConfig {
        /// Read root from env (`PARTKEEPR_STORAGE_DIR`), falling back to
        /// `~/.local/share/partkeepr-ng/files/`. Creates the directory if
        /// missing.
        pub async fn from_env_and_init() -> anyhow::Result<Self> {
            let root = match env::var("PARTKEEPR_STORAGE_DIR") {
                Ok(v) => PathBuf::from(v),
                Err(_) => {
                    let home = env::var("HOME").context("HOME not set")?;
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join("partkeepr-ng")
                        .join("files")
                }
            };
            fs::create_dir_all(&root)
                .await
                .with_context(|| format!("create storage root {}", root.display()))?;
            tracing::info!(root = %root.display(), "file storage root");
            Ok(Self {
                root,
                clamd: ClamdConfig::from_env(),
            })
        }
    }
    
    /// Bytes + metadata extracted from an inbound multipart field. The
    /// caller pulls these out of axum's `Multipart`.
    pub struct UploadInput {
        pub bytes: Bytes,
        pub original_filename: Option<String>,
        pub content_type: Option<String>,
    }
    
    /// Result of a successful save — exactly the columns the per-entity
    /// attachment tables need (`filename`, `originalname`, `mimetype`,
    /// `size`, `isImage`, `extension`).
    pub struct SavedFile {
        pub uuid: String,
        pub original_filename: String,
        pub mimetype: String,
        pub size: i32,
        pub extension: Option<String>,
        pub is_image: bool,
    }
    
    #[derive(Debug)]
    pub enum SaveError {
        Infected(String),
        Io(std::io::Error),
        Other(anyhow::Error),
    }
    
    impl From<std::io::Error> for SaveError {
        fn from(e: std::io::Error) -> Self { SaveError::Io(e) }
    }
    impl From<anyhow::Error> for SaveError {
        fn from(e: anyhow::Error) -> Self { SaveError::Other(e) }
    }
    
    /// Save an uploaded file under `{root}/{table}/{uuid}.{ext}`. If the
    /// file is an image (per its content type), also generates a 200x200
    /// JPEG thumbnail at `{uuid}.thumb.jpg`. If clamd is configured, the
    /// bytes are scanned before anything hits disk.
    pub async fn save_upload(
        cfg: &StorageConfig,
        table: &str,
        input: UploadInput,
    ) -> Result<SavedFile, SaveError> {
        // Virus scan first — bail before touching the disk.
        if cfg.clamd.enabled() {
            match clamd::scan(&cfg.clamd, &input.bytes).await {
                Ok(ScanResult::Clean) => {}
                Ok(ScanResult::Infected(sig)) => return Err(SaveError::Infected(sig)),
                Err(e) => {
                    // Treat scanner connectivity errors as fatal — we
                    // would rather refuse uploads than silently bypass
                    // the scanner when the env var said it should run.
                    return Err(SaveError::Other(e));
                }
            }
        }
    
        let original_filename = input
            .original_filename
            .unwrap_or_else(|| "upload".to_string());
        let extension = extension_from_filename(&original_filename);
        let mimetype = input
            .content_type
            .or_else(|| {
                mime_guess::from_path(&original_filename)
                    .first()
                    .map(|m| m.essence_str().to_string())
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let is_image = mimetype.starts_with("image/");
        let size = input.bytes.len() as i32;
    
        let uuid = uuid::Uuid::new_v4().to_string();
        let dir = cfg.root.join(table);
        fs::create_dir_all(&dir).await?;
        let main_path = file_path(&dir, &uuid, extension.as_deref());
    
        // Write atomically: write to .tmp then rename, so a crash mid-write
        // doesn't leave a half-file referenced by the DB row.
        let tmp = main_path.with_extension("tmp");
        let mut f = fs::File::create(&tmp).await?;
        f.write_all(&input.bytes).await?;
        f.sync_all().await?;
        drop(f);
        fs::rename(&tmp, &main_path).await?;
    
        if is_image {
            if let Err(e) = generate_thumbnail(&main_path, &thumbnail_path(&dir, &uuid)).await {
                tracing::warn!(error = %e, original = %original_filename, "thumbnail generation failed");
                // Non-fatal: keep the original.
            }
        }
    
        Ok(SavedFile {
            uuid,
            original_filename,
            mimetype,
            size,
            extension,
            is_image,
        })
    }
    
    /// Build the on-disk path for the original of `{uuid}` in `dir`. We
    /// preserve the extension when present so the file is recognisable on
    /// disk (and so that `Content-Disposition` can fall back to it cleanly).
    pub fn file_path(dir: &Path, uuid: &str, ext: Option<&str>) -> PathBuf {
        match ext {
            Some(e) if !e.is_empty() => dir.join(format!("{uuid}.{e}")),
            _ => dir.join(uuid),
        }
    }
    
    /// Like `file_path`, but resolves the actual file on disk even when the
    /// DB's `extension` column doesn't match what's there. Falls back to a
    /// directory glob so legacy rows with `extension = NULL` (e.g. our
    /// imported ManufacturerICLogo / FootprintImage data) still serve.
    ///
    /// Probes in order:
    /// 1. `{uuid}.{hint_ext}` — fast path, single stat.
    /// 2. `{uuid}` — extensionless legacy convention.
    /// 3. Walk `dir` for an entry whose name starts with `{uuid}.` and
    ///    isn't the thumbnail companion. Slow, but only hit for rows where
    ///    1+2 missed.
    pub async fn resolve_file_path(
        dir: &Path,
        uuid: &str,
        hint_ext: Option<&str>,
    ) -> Option<PathBuf> {
        if let Some(e) = hint_ext {
            if !e.is_empty() {
                let p = dir.join(format!("{uuid}.{e}"));
                if tokio::fs::try_exists(&p).await.unwrap_or(false) {
                    return Some(p);
                }
            }
        }
        let plain = dir.join(uuid);
        if tokio::fs::try_exists(&plain).await.unwrap_or(false) {
            return Some(plain);
        }
        let prefix = format!("{uuid}.");
        let thumb_marker = ".thumb.";
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&prefix) && !name.contains(thumb_marker) {
                        return Some(entry.path());
                    }
                }
            }
        }
        None
    }
    
    pub fn thumbnail_path(dir: &Path, uuid: &str) -> PathBuf {
        dir.join(format!("{uuid}.thumb.jpg"))
    }
    
    /// Extract the lowercased extension (sans dot) from a filename. Returns
    /// None if no `.` or trailing-dot.
    fn extension_from_filename(name: &str) -> Option<String> {
        let dot = name.rfind('.')?;
        let ext = &name[dot + 1..];
        if ext.is_empty() || ext.contains('/') {
            None
        } else {
            Some(ext.to_ascii_lowercase())
        }
    }
    
    /// Public alias — lets the attachment handler regenerate a thumbnail
    /// lazily when it's missing on disk (typical for legacy-imported
    /// images where the original PartKeepr never produced a thumb).
    pub async fn regenerate_thumbnail(src: &Path, out: &Path) -> anyhow::Result<()> {
        generate_thumbnail(src, out).await
    }
    
    /// Generate a 200x200 JPEG thumbnail at `out`. Reads the source from
    /// `src`. CPU-bound; runs on the tokio blocking pool.
    async fn generate_thumbnail(src: &Path, out: &Path) -> anyhow::Result<()> {
        let src = src.to_path_buf();
        let out = out.to_path_buf();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let img = ImageReader::open(&src)?
                .with_guessed_format()?
                .decode()?;
            let thumb = img.thumbnail(200, 200);
            let mut buf = Cursor::new(Vec::with_capacity(8192));
            thumb.write_to(&mut buf, image::ImageFormat::Jpeg)?;
            std::fs::write(&out, buf.into_inner())?;
            Ok(())
        })
        .await??;
        Ok(())
    }
    // SPDX-License-Identifier: GPL-3.0-or-later
    
    //! Slice 7: per-entity attachment endpoints.
    //!
    //! Five Doctrine entities — PartAttachment, FootprintImage,
    //! FootprintAttachment, ManufacturerICLogo, StorageLocationImage —
    //! all share the same overall shape (id, {parent}_id, filename,
    //! originalname, mimetype, size, description, extension, created),
    //! and they all live under their PascalCase subdir on disk.
    //!
    //! The wire URL space is split:
    //!
    //! - **POST** `/api/{parent}/:id/{children}` — upload (multipart);
    //!   parent path varies per entity.
    //! - **GET** `/files/:kind/:id` — download original (browser-friendly,
    //!   no `/api` prefix so HTML can `<img src=>` directly).
    //! - **GET** `/files/:kind/:id/thumb` — JPEG thumbnail (image kinds
    //!   only; 404 for non-image rows).
    //! - **PUT** `/api/attachments/:kind/:id` — update description.
    //! - **DELETE** `/api/attachments/:kind/:id` — remove file + DB row.
    //!
    //! `:kind` is the Doctrine class name (PascalCase singular).
    
    use axum::{
        extract::{Multipart, Path, State},
        http::{header, HeaderMap, StatusCode},
        response::{IntoResponse, Response},
        Json,
    };
    use bytes::{Bytes, BytesMut};
    use futures_util::StreamExt;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use sqlx::MySqlPool;
    use tokio::fs;
    use tokio::io::AsyncReadExt;
    
    use crate::error::AppError;
    use crate::handlers::storage::{self, SaveError, StorageConfig, UploadInput};
    
    // ---------------------------------------------------------------------------
    //  Kind enum + per-kind metadata
    // ---------------------------------------------------------------------------
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AttachmentKind {
        PartAttachment,
        FootprintImage,
        FootprintAttachment,
        ManufacturerICLogo,
        StorageLocationImage,
        ProjectAttachment,
    }
    
    impl AttachmentKind {
        pub fn from_str(s: &str) -> Option<Self> {
            Some(match s {
                "PartAttachment"       => Self::PartAttachment,
                "FootprintImage"       => Self::FootprintImage,
                "FootprintAttachment"  => Self::FootprintAttachment,
                "ManufacturerICLogo"   => Self::ManufacturerICLogo,
                "StorageLocationImage" => Self::StorageLocationImage,
                "ProjectAttachment"    => Self::ProjectAttachment,
                _ => return None,
            })
        }
    
        /// PascalCase entity name. Doubles as on-disk subdir and Doctrine
        /// `type` discriminator value.
        pub fn name(self) -> &'static str {
            match self {
                Self::PartAttachment       => "PartAttachment",
                Self::FootprintImage       => "FootprintImage",
                Self::FootprintAttachment  => "FootprintAttachment",
                Self::ManufacturerICLogo   => "ManufacturerICLogo",
                Self::StorageLocationImage => "StorageLocationImage",
                Self::ProjectAttachment    => "ProjectAttachment",
            }
        }
    
        /// SQL table name (= entity name; Doctrine uses class-name tables).
        fn table(self) -> &'static str { self.name() }
    
        /// FK column linking the attachment row to its parent entity.
        fn parent_fk(self) -> &'static str {
            match self {
                Self::PartAttachment       => "part_id",
                Self::FootprintImage       => "footprint_id",
                Self::FootprintAttachment  => "footprint_id",
                Self::ManufacturerICLogo   => "manufacturer_id",
                Self::StorageLocationImage => "storageLocation_id",
                Self::ProjectAttachment    => "project_id",
            }
        }
    
        /// Whether the table has the `isImage` column (only PartAttachment
        /// does — the other tables imply by-class).
        fn has_is_image_column(self) -> bool {
            matches!(self, Self::PartAttachment)
        }
    
        /// Whether attachments of this kind are *always* images. Determines
        /// what we set in `isImage` on insert (for PartAttachment) and is
        /// also used to gate thumbnail generation when the upload's MIME
        /// turns out to be ambiguous.
        fn always_image(self) -> bool {
            matches!(self, Self::FootprintImage | Self::ManufacturerICLogo | Self::StorageLocationImage)
        }
    }
    
    // ---------------------------------------------------------------------------
    //  Wire shapes
    // ---------------------------------------------------------------------------
    
    #[derive(Debug, Serialize, sqlx::FromRow)]
    pub struct AttachmentRow {
        pub id: i32,
        pub filename: String,
        pub originalname: Option<String>,
        pub mimetype: String,
        pub size: i32,
        pub description: Option<String>,
        pub extension: Option<String>,
        pub is_image: bool,
    }
    
    #[derive(Debug, Deserialize)]
    pub struct UpdateDescription {
        pub description: Option<String>,
    }
    
    // ---------------------------------------------------------------------------
    //  Upload (multipart) — per-entity entry points wrap a shared core
    // ---------------------------------------------------------------------------
    
    pub async fn upload_part(
        State(pool): State<MySqlPool>,
        State(store): State<StorageConfig>,
        Path(parent_id): Path<i32>,
        multipart: Multipart,
    ) -> Result<Json<Vec<AttachmentRow>>, AppError> {
        upload_core(&pool, &store, AttachmentKind::PartAttachment, parent_id, multipart).await
    }
    
    pub async fn upload_footprint_image(
    rustc 1.86.0 (05f9846f8 2025-03-31)
    1048:name = "image"
    2416:name = "time"
    ✓ • 129ms

## Assistant

Your database dump and archive are both present. I found an important mismatch in the migration guide: the current backend expects images and attachments together under entity-named folders, while your archive uses the original separate `data/files` and `data/images` trees. I’m checking the folder names so the import instructions account for that.

## Activity

    $ python3 - <<'PY'
        import tarfile, collections
        counts=collections.Counter()
        with tarfile.open('/home/alan/partkeepr-files.tar.gz','r|gz') as t:
         for m in t:
          if m.isfile(): counts['/'.join(m.name.split('/')[:-1])]+=1
        for p,n in sorted(counts.items()): print(n,p)
        PY
    1 data/files/FootprintAttachment
    232 data/files/PartAttachment
    231 data/files/Temporary
    37 data/images/footprint
    346 data/images/iclogo
    2 data/images/storagelocation
    5 data/temp
    ✓ • 1.92s

## Assistant

Yes—your dump and archive contain what you need. I confirmed both exist:

- Database: `/tmp/partkeepr-prod.sql`
- Files: `/home/alan/partkeepr-files.tar.gz`

Use a **separate database and copied files** for partkeepr-ng, keeping your original installation available during testing. The instructions below install directly on this Debian host.

One correction to the repository’s migration guide: **the backend currently requires a different image directory layout.** Simply extracting your archive and setting `PARTKEEPR_STORAGE_DIR` will leave images inaccessible. The commands below handle that.

**1. Prepare the software**

You need MariaDB **10.6 or newer**, a current stable Rust toolchain, and build tools. Your installed Rust is 1.86.0; update it before building.

```bash
sudo apt update
sudo apt install mariadb-server mariadb-client build-essential pkg-config openssl rsync

rustup update stable

cd /home/alan/partkeepr-ng
cargo +stable build --locked --release --bin partkeepr-api
```

The frontend is already included; it needs no separate build.

**2. Restore your database into a new database**

Choose a database password. A long alphanumeric password avoids URL-encoding complications later.

```bash
sudo mariadb
```

Run:

```sql
CREATE DATABASE partkeepr_ng
  CHARACTER SET utf8 COLLATE utf8_unicode_ci;

CREATE USER 'partkeepr_ng'@'localhost'
  IDENTIFIED BY 'REPLACE_WITH_YOUR_PASSWORD';

GRANT ALL PRIVILEGES ON partkeepr_ng.*
  TO 'partkeepr_ng'@'localhost';

EXIT;
```

Import your dump:

```bash
mysql -u partkeepr_ng -p partkeepr_ng < /tmp/partkeepr-prod.sql
```

Your dump contains no `CREATE DATABASE` or `USE` statements, so this command targets the new database.

Check the restored counts:

```bash
mysql -u partkeepr_ng -p partkeepr_ng -e "
SELECT COUNT(*) AS parts FROM Part;
SELECT COUNT(*) AS attachments FROM PartAttachment;
SELECT COUNT(*) AS stock_entries FROM StockEntry;
"
```

**3. Apply the migrations once**

From the repository:

```bash
bash scripts/migrate-from-upstream.sh \
  -u partkeepr_ng \
  -p 'REPLACE_WITH_YOUR_PASSWORD' \
  -d partkeepr_ng
```

There are six migrations. Despite the guide calling them all additive, migration `0005` also changes certain part-unit assignments and deletes specific reel/cut-tape unit rows. This is another reason to work on a copy.

If a migration fails, stop and inspect the error. SQL files are **not rolled back as a whole**, and the final migration is not safe to rerun unchanged after partial success.

**4. Extract and arrange your attachments and images**

These commands use directories owned by your login user, suitable for the initial trial:

```bash
mkdir -p /home/alan/partkeepr-ng-import
mkdir -p /home/alan/.local/share/partkeepr-ng/files

tar --no-same-owner -xzf /home/alan/partkeepr-files.tar.gz \
  -C /home/alan/partkeepr-ng-import
```

Your archive contains 232 part attachments, one footprint attachment, 37 footprint images, 346 manufacturer logos, and two storage-location images.

Copy them into the layout the backend actually uses:

```bash
rsync -a /home/alan/partkeepr-ng-import/data/files/PartAttachment \
  /home/alan/.local/share/partkeepr-ng/files/

rsync -a /home/alan/partkeepr-ng-import/data/files/FootprintAttachment \
  /home/alan/.local/share/partkeepr-ng/files/

rsync -a /home/alan/partkeepr-ng-import/data/images/footprint/ \
  /home/alan/.local/share/partkeepr-ng/files/FootprintImage/

rsync -a /home/alan/partkeepr-ng-import/data/images/iclogo/ \
  /home/alan/.local/share/partkeepr-ng/files/ManufacturerICLogo/

rsync -a /home/alan/partkeepr-ng-import/data/images/storagelocation/ \
  /home/alan/.local/share/partkeepr-ng/files/StorageLocationImage/
```

Keep the filenames unchanged: they connect the files to database records. The temporary files remain in your extracted backup and are not needed by the new application. Thumbnails are generated when requested.

**5. Configure and start**

```bash
cd /home/alan/partkeepr-ng
cp backend/.env.example backend/.env
openssl rand -hex 48
```

Edit `backend/.env`, setting:

```dotenv
DATABASE_URL=mysql://partkeepr_ng:REPLACE_WITH_YOUR_PASSWORD@localhost:3306/partkeepr_ng
PARTKEEPR_SESSION_SECRET=PASTE_THE_GENERATED_HEX_STRING
PARTKEEPR_STORAGE_DIR=/home/alan/.local/share/partkeepr-ng/files
PARTKEEPR_FRONTEND_DIR=/home/alan/partkeepr-ng/frontend
BIND_ADDR=127.0.0.1:8080
```

Then:

```bash
chmod 600 backend/.env
cd /home/alan/partkeepr-ng/backend
../target/release/partkeepr-api
```

Open **http://localhost:8080/** and use your existing PartKeepr credentials.

Verify parts, stock quantities, categories, PDF downloads, footprint images, and manufacturer logos before switching over. This runs in the foreground; a systemd service can follow once the imported installation checks out. The repository labels this version alpha.

I inspected your inputs and the implementation; I haven’t imported the database or changed your installation.

## User

Can you proceed with the installation? If there is some additional data from the origian PartKeepr install that I need to extract, please ask

## Assistant

I’ll install against a new database and copy your files, then build and verify the application. I’ll also check whether the dump references any files missing from your archive.

## Activity

    $ docker ps --format '{{.Names}} {{.Image}} {{.Ports}}'; command -v mariadbd mysqld gcc pkg-config; rustc --version; ls -l /var/run/mysqld; sed -n '60,120p' backend/src/main.rs; rg -n 'CREATE TABLE|filename|extension' /tmp/partkeepr-prod.sql | head -35
    permission denied while trying to connect to the docker API at unix:///var/run/docker.sock
    /usr/sbin/mysqld
    /usr/bin/gcc
    /usr/bin/pkg-config
    rustc 1.86.0 (05f9846f8 2025-03-31)
    total 12
    -rw-r----- 1 nobody nogroup 5 Sep  3 16:42 mysqld.pid
    srwxrwxrwx 1 nobody nogroup 0 Sep  3 16:42 mysqld.sock
    -rw------- 1 nobody nogroup 5 Sep  3 16:42 mysqld.sock.lock
    srwxrwxrwx 1 nobody nogroup 0 Sep  3 16:42 mysqlx.sock
    -rw------- 1 nobody nogroup 5 Sep  3 16:42 mysqlx.sock.lock
        fn from_ref(s: &AppState) -> Self { s.jlcparts.clone() }
    }
    
    #[tokio::main]
    async fn main() -> anyhow::Result<()> {
        dotenvy::dotenv().ok();
    
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info,partkeepr_api=debug,tower_http=debug")),
            )
            .init();
    
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set (see .env.example)")?;
        let bind_addr: SocketAddr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse()
            .context("BIND_ADDR must be a valid socket address")?;
    
        let pool = MySqlPoolOptions::new()
            .max_connections(8)
            .connect(&database_url)
            .await
            .context("connecting to database")?;
        tracing::info!("connected to database");
    
        // Startup invariants — schema sanity + seed rows that the app
        // assumes exist. See backend/src/startup.rs for the list.
        startup::ensure_invariants(&pool).await?;
    
        let store = StorageConfig::from_env_and_init().await?;
        let signer = SessionSigner::from_env()?;
        let printing = handlers::printing::PrintConfig::from_env();
        let mouser = handlers::mouser::MouserConfig::from_env();
        let digikey = handlers::digikey::DigiKeyConfig::from_env();
        let trustedparts = handlers::trustedparts::TrustedPartsConfig::from_env();
        let jlcparts = handlers::jlcparts::JlcPartsConfig::from_env().await;
        let state = AppState { pool, store, signer, printing, mouser, digikey, trustedparts, jlcparts };
    
        // One-line summary of optional-integration state. Operators that
        // miss the per-handler INFO/WARN lines above can grep this banner
        // for `=off` to find integrations they may want to enable in
        // /etc/conf.d/partkeepr-ng (or backend/.env in dev).
        let on_off = |b: bool| if b { "on" } else { "off" };
        tracing::info!(
            "feature status — clamd={} ptouch={} mouser_search={} mouser_orders={} \
             digikey={} trustedparts={} jlcparts={}",
            on_off(state.store.clamd.enabled()),
            on_off(state.printing.ptouch_bin.is_some()),
            on_off(state.mouser.search_key.is_some()),
            on_off(state.mouser.order_key.is_some()),
            on_off(state.digikey.available()),
            on_off(state.trustedparts.available()),
            on_off(state.jlcparts.available()),
        );
    
        // Frontend is served as static assets out of this directory. Default
        // is "../frontend" relative to the backend cwd (matches the dev
        // checkout layout). Production sets PARTKEEPR_FRONTEND_DIR to the
    26:CREATE TABLE `BatchJob` (
    51:CREATE TABLE `BatchJobQueryField` (
    81:CREATE TABLE `BatchJobUpdateField` (
    110:CREATE TABLE `CachedImage` (
    135:CREATE TABLE `CronLogger` (
    161:CREATE TABLE `Distributor` (
    194:CREATE TABLE `FOSUser` (
    234:CREATE TABLE `Footprint` (
    263:CREATE TABLE `FootprintAttachment` (
    267:  `filename` varchar(255) NOT NULL,
    271:  `extension` varchar(255) DEFAULT NULL,
    297:CREATE TABLE `FootprintCategory` (
    332:CREATE TABLE `FootprintImage` (
    336:  `filename` varchar(255) NOT NULL,
    340:  `extension` varchar(255) DEFAULT NULL,
    366:CREATE TABLE `GridPreset` (
    394:CREATE TABLE `ImportPreset` (
    420:CREATE TABLE `Manufacturer` (
    451:CREATE TABLE `ManufacturerICLogo` (
    455:  `filename` varchar(255) NOT NULL,
    459:  `extension` varchar(255) DEFAULT NULL,
    485:CREATE TABLE `MetaPartParameterCriteria` (
    522:CREATE TABLE `Part` (
    572:CREATE TABLE `PartAttachment` (
    576:  `filename` varchar(255) NOT NULL,
    580:  `extension` varchar(255) DEFAULT NULL,
    607:CREATE TABLE `PartCategory` (
    642:CREATE TABLE `PartDistributor` (
    677:CREATE TABLE `PartKeeprUser` (
    712:CREATE TABLE `PartManufacturer` (
    742:CREATE TABLE `PartParameter` (
    790:CREATE TABLE `PartUnit` (
    816:CREATE TABLE `Project` (
    843:CREATE TABLE `ProjectAttachment` (
    847:  `filename` varchar(255) NOT NULL,
    ✓ • 0ms

## Activity

    $ ls -l scripts/migrate-from-upstream.sh; cat .gitignore; docker compose version; ss -ltn '( sport = :8080 or sport = :3307 )'
    -rwxr-xr-x 1 alan alan 4393 Oct 16 06:24 scripts/migrate-from-upstream.sh
    # Build outputs (Cargo puts the workspace target/ at the root; per-crate
    # target/ dirs may also exist if a crate was built standalone)
    target/
    backend/target/
    frontend/target/
    frontend/dist/
    
    # Dev secrets and local-only env
    backend/.env
    frontend/.env
    
    # Upstream reference clone — re-clone via:
    #   cd reference && git clone https://github.com/partkeepr/PartKeepr.git
    # (gitignored to avoid embedding a foreign git repo and to keep our
    # tree small. The 1272-issue archive in docs/upstream-issues/ is
    # tracked because it preserves data that may not be reachable on
    # GitHub forever.)
    reference/PartKeepr/
    
    # Live data dumps — user's actual inventory; not for source control.
    # The schema-only dump and migration list ARE tracked so anyone can
    # spin up a dev DB from a checkout.
    docs/live-schema/partkeepr-data.sql
    
    # Editor / OS junk
    **/*.swp
    **/*.swo
    .DS_Store
    *~
    Docker Compose version v2.24.6
    State  Recv-Q Send-Q Local Address:Port Peer Address:PortProcess
    LISTEN 0      511                *:8080            *:*
    ✓ • 0ms

## Assistant

The original PartKeepr containers are still running, so I’ll create a separate MariaDB container for partkeepr-ng. Port 8080 is already occupied; I’ll use port 8082, bound to localhost for the initial installation. Your archive has the expected attachment and image folders, and I’ll compare them with the restored database records.

## Activity

    $ docker ps --format '{{.Names}} {{.Image}} {{.Ports}}'; systemctl is-active mariadb mysql; rustup update stable
    vaultwarden vaultwarden/server:latest 127.0.0.1:8787->80/tcp
    healthchecks healthchecks/healthchecks:latest 0.0.0.0:19659->8000/tcp, [::]:19659->8000/tcp
    shotcandy-shotcandy-1 shotcandy-shotcandy 0.0.0.0:48271->80/tcp, [::]:48271->80/tcp
    livesync-cloudflared cloudflare/cloudflared:latest
    livesync-couchdb couchdb:latest 4369/tcp, 9100/tcp, 0.0.0.0:5984->5984/tcp, [::]:5984->5984/tcp
    rss-bridge rssbridge/rss-bridge:latest 0.0.0.0:4242->80/tcp, [::]:4242->80/tcp
    opengist ghcr.io/thomiceli/opengist:1 2222/tcp, 6158/tcp, 0.0.0.0:6157->6157/tcp, [::]:6157->6157/tcp
    vw-caddy caddy:2 80/tcp, 443/tcp, 2019/tcp, 443/udp, 10.0.0.1:6602->6602/tcp
    inventree-proxy caddy:alpine 443/udp, 0.0.0.0:443->443/tcp, [::]:443->443/tcp, 2019/tcp, 0.0.0.0:5462->80/tcp, [::]:5462->80/tcp
    inventree-worker inventree/inventree:stable 8000/tcp
    inventree-server inventree/inventree:stable 8000/tcp
    inventree-cache redis:7-alpine 6379/tcp
    inventree-db postgres:17 5432/tcp
    partkeepr-partkeepr-db-1 mariadb:10.5 3306/tcp
    partkeepr-partkeepr-1 f0rc3/partkeepr:latest 9000/tcp, 0.0.0.0:5461->80/tcp, [::]:5461->80/tcp
    languagetool meyay/languagetool:latest 0.0.0.0:8081->8081/tcp, [::]:8081->8081/tcp
    inactive
    active
    info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'
    info: latest update on 2026-10-15, rust version 1.99.1 (58cf8f9c3 2026-10-13)
    info: downloading component 'cargo'
    info: downloading component 'clippy'
    info: downloading component 'rust-docs'
    info: downloading component 'rust-std'
    info: downloading component 'rustc'
    info: downloading component 'rustfmt'
    info: removing previous version of component 'cargo'
    info: removing previous version of component 'clippy'
    info: removing previous version of component 'rust-docs'
    info: removing previous version of component 'rust-std'
    info: removing previous version of component 'rustc'
    info: removing previous version of component 'rustfmt'
    info: installing component 'cargo'
    info: installing component 'clippy'
    info: installing component 'rust-docs'
    info: installing component 'rust-std'
    info: installing component 'rustc'
    info: installing component 'rustfmt'
    
      stable-x86_64-unknown-linux-gnu updated - rustc 1.99.1 (58cf8f9c3 2026-10-13) (from rustc 1.86.0 (05f9846f8 2025-03-31))
    
    info: checking for self-update
    info: downloading self-update
    ✓ • 29.42s

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import secrets
        p=Path('local-install');p.mkdir(mode=0o700,exist_ok=True)
        (p/'.gitignore').write_text('*\n')
        for d in ['storage','original','bin']: (p/d).mkdir(exist_ok=True)
        env=p/'database.env'
        if not env.exists():
         env.write_text('MARIADB_DATABASE=partkeepr_ng\nMARIADB_USER=partkeepr_ng\nMARIADB_PASSWORD='+secrets.token_hex(24)+'\nMARIADB_ROOT_PASSWORD='+secrets.token_hex(24)+'\n');env.chmod(0o600)
        values=dict(line.split('=',1) for line in env.read_text().splitlines())
        a=p/'app.env'
        if not a.exists():
         a.write_text('DATABASE_URL=mysql://partkeepr_ng:'+values['MARIADB_PASSWORD']+'@db:3306/partkeepr_ng\nPARTKEEPR_SESSION_SECRET='+secrets.token_hex(48)+'\nPARTKEEPR_STORAGE_DIR=/storage\nPARTKEEPR_FRONTEND_DIR=/frontend\nBIND_ADDR=0.0.0.0:8080\nRUST_LOG=info\n');a.chmod(0o600)
        (p/'compose.yaml').write_text('''name: partkeepr-ng
        services:
          db:
            image: mariadb:10.11
            restart: unless-stopped
            env_file: database.env
            volumes:
              - database:/var/lib/mysql
            healthcheck:
              test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
              interval: 5s
              timeout: 5s
              retries: 30
          app:
            build:
              context: .
              dockerfile: Dockerfile
            restart: unless-stopped
            user: "1000:1000"
            env_file: app.env
            ports:
              - "127.0.0.1:8082:8080"
            volumes:
              - ./storage:/storage
              - ../frontend:/frontend:ro
            depends_on:
              db:
                condition: service_healthy
        volumes:
          database:
        ''')
        (p/'Dockerfile').write_text('''FROM debian:bookworm-slim
        RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
        COPY bin/partkeepr-api /usr/local/bin/partkeepr-api
        ENTRYPOINT ["/usr/local/bin/partkeepr-api"]
        ''')
        print('Created private installation configuration without displaying credentials.')
        PY
    Created private installation configuration without displaying credentials.
    ✓ • 33ms

## Activity

    $ python3 - <<'PY'
        import tarfile, shutil
        from pathlib import Path
        root=Path('/home/alan/partkeepr-ng/local-install')
        mapping={'data/files/PartAttachment':'PartAttachment','data/files/FootprintAttachment':'FootprintAttachment','data/files/ProjectAttachment':'ProjectAttachment','data/images/footprint':'FootprintImage','data/images/iclogo':'ManufacturerICLogo','data/images/storagelocation':'StorageLocationImage'}
        counts={}
        with tarfile.open('/home/alan/partkeepr-files.tar.gz','r|gz') as archive:
         for m in archive:
          if not m.isfile(): continue
          path=Path(m.name)
          parent=str(path.parent)
          if parent not in mapping: continue
          dst=root/'storage'/mapping[parent]/path.name
          dst.parent.mkdir(exist_ok=True)
          with archive.extractfile(m) as src, dst.open('wb') as out: shutil.copyfileobj(src,out)
          counts[mapping[parent]]=counts.get(mapping[parent],0)+1
        print(counts)
        PY
    {'FootprintAttachment': 1, 'PartAttachment': 232, 'FootprintImage': 37, 'ManufacturerICLogo': 346, 'StorageLocationImage': 2}
    ✓ • 2.72s

## Activity

    $ docker compose -f local-install/compose.yaml up -d db
     db Pulling
     c34808af6b3e Pulling fs layer
     4ef363dc51e7 Pulling fs layer
     59013e7d205a Pulling fs layer
     37f986522545 Pulling fs layer
     3e968919cb92 Pulling fs layer
     98c4455a9898 Pulling fs layer
     d626f8bd756e Pulling fs layer
     e7908f300846 Pulling fs layer
     4ef363dc51e7 Downloading [>                                                  ]  1.049MB/68.83MB
     4ef363dc51e7 Downloading [>                                                  ]  1.049MB/68.83MB
     4ef363dc51e7 Downloading [=>                                                 ]  2.097MB/68.83MB
     4ef363dc51e7 Downloading [=>                                                 ]  2.097MB/68.83MB
     98c4455a9898 Downloading [=>                                                 ]  1.049MB/29.75MB
     98c4455a9898 Downloading [=>                                                 ]  1.049MB/29.75MB
     4ef363dc51e7 Downloading [==>                                                ]  3.146MB/68.83MB
     98c4455a9898 Downloading [===>                                               ]  2.097MB/29.75MB
     37f986522545 Downloading [=========>                                         ]  1.049MB/5.566MB
     4ef363dc51e7 Downloading [==>                                                ]  3.146MB/68.83MB
     37f986522545 Downloading [=========>                                         ]  1.049MB/5.566MB
     4ef363dc51e7 Downloading [==>                                                ]  3.146MB/68.83MB
     98c4455a9898 Downloading [===>                                               ]  2.097MB/29.75MB
     98c4455a9898 Downloading [=====>                                             ]  3.146MB/29.75MB
     37f986522545 Downloading [=========>                                         ]  1.049MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     37f986522545 Downloading [==================>                                ]  2.097MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     98c4455a9898 Downloading [=======>                                           ]  4.194MB/29.75MB
     37f986522545 Downloading [==================>                                ]  2.097MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     98c4455a9898 Downloading [=======>                                           ]  4.194MB/29.75MB
     98c4455a9898 Downloading [========>                                          ]  5.243MB/29.75MB
     37f986522545 Downloading [==================>                                ]  2.097MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     37f986522545 Downloading [==================>                                ]  2.097MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     98c4455a9898 Downloading [==========>                                        ]  6.291MB/29.75MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     98c4455a9898 Downloading [==========>                                        ]  6.291MB/29.75MB
     37f986522545 Downloading [============================>                      ]  3.146MB/5.566MB
     98c4455a9898 Downloading [============>                                      ]   7.34MB/29.75MB
     37f986522545 Downloading [============================>                      ]  3.146MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     37f986522545 Downloading [=====================================>             ]  4.194MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  4.194MB/68.83MB
     98c4455a9898 Downloading [==============>                                    ]  8.389MB/29.75MB
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [==============>                                    ]  8.389MB/29.75MB
     37f986522545 Downloading [=====================================>             ]  4.194MB/5.566MB
     37f986522545 Downloading [=====================================>             ]  4.194MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [===============>                                   ]  9.437MB/29.75MB
     37f986522545 Downloading [===============================================>   ]  5.243MB/5.566MB
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [=================>                                 ]  10.49MB/29.75MB
     37f986522545 Download complete
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [=================>                                 ]  10.49MB/29.75MB
     59013e7d205a Download complete
     98c4455a9898 Downloading [===================>                               ]  11.53MB/29.75MB
     3e968919cb92 Download complete
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [=====================>                             ]  12.58MB/29.75MB
     c34808af6b3e Download complete
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     d626f8bd756e Download complete
     e7908f300846 Download complete
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     98c4455a9898 Downloading [======================>                            ]  13.63MB/29.75MB
     98c4455a9898 Downloading [========================>                          ]  14.68MB/29.75MB
     4ef363dc51e7 Downloading [===>                                               ]  5.243MB/68.83MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [==========================>                        ]  15.73MB/29.75MB
     c3ef19ce92a2 Download complete
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [============================>                      ]  16.78MB/29.75MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [=============================>                     ]  17.83MB/29.75MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [===============================>                   ]  18.87MB/29.75MB
     98c4455a9898 Downloading [===============================>                   ]  18.87MB/29.75MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [=================================>                 ]  19.92MB/29.75MB
     110b02691b4f Downloading [==========>                                        ]  1.049MB/4.787MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [===================================>               ]  20.97MB/29.75MB
     110b02691b4f Downloading [==========>                                        ]  1.049MB/4.787MB
     110b02691b4f Downloading [==========>                                        ]  1.049MB/4.787MB
     4ef363dc51e7 Downloading [====>                                              ]  6.291MB/68.83MB
     98c4455a9898 Downloading [===================================>               ]  20.97MB/29.75MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [=====================================>             ]  22.02MB/29.75MB
     110b02691b4f Downloading [=====================>                             ]  2.097MB/4.787MB
     110b02691b4f Downloading [=====================>                             ]  2.097MB/4.787MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [======================================>            ]  23.07MB/29.75MB
     98c4455a9898 Downloading [======================================>            ]  23.07MB/29.75MB
     110b02691b4f Downloading [=====================>                             ]  2.097MB/4.787MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     110b02691b4f Downloading [================================>                  ]  3.146MB/4.787MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [========================================>          ]  24.12MB/29.75MB
     110b02691b4f Downloading [================================>                  ]  3.146MB/4.787MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [========================================>          ]  24.12MB/29.75MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [==========================================>        ]  25.17MB/29.75MB
     110b02691b4f Downloading [===========================================>       ]  4.194MB/4.787MB
     110b02691b4f Downloading [===========================================>       ]  4.194MB/4.787MB
     4ef363dc51e7 Downloading [=====>                                             ]   7.34MB/68.83MB
     98c4455a9898 Downloading [==========================================>        ]  25.17MB/29.75MB
     4ef363dc51e7 Downloading [======>                                            ]  8.389MB/68.83MB
     98c4455a9898 Downloading [============================================>      ]  26.21MB/29.75MB
     110b02691b4f Download complete
     98c4455a9898 Downloading [=============================================>     ]  27.26MB/29.75MB
     4ef363dc51e7 Downloading [======>                                            ]  8.389MB/68.83MB
     4ef363dc51e7 Downloading [======>                                            ]  8.389MB/68.83MB
     98c4455a9898 Downloading [===============================================>   ]  28.31MB/29.75MB
     4ef363dc51e7 Downloading [======>                                            ]  8.389MB/68.83MB
     98c4455a9898 Downloading [==================================================>]  29.75MB/29.75MB
     4ef363dc51e7 Downloading [======>                                            ]  9.437MB/68.83MB
     98c4455a9898 Download complete
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [=======>                                           ]  10.49MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [========>                                          ]  11.53MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [=========>                                         ]  12.58MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [=========>                                         ]  13.63MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [==========>                                        ]  14.68MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [===========>                                       ]  15.73MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [============>                                      ]  16.78MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [=============>                                     ]  18.87MB/68.83MB
     98c4455a9898 Extracting 1 s
     4ef363dc51e7 Downloading [==============>                                    ]  19.92MB/68.83MB
     c34808af6b3e Extracting 1 s
     98c4455a9898 Pull complete
     4ef363dc51e7 Downloading [===============>                                   ]  20.97MB/68.83MB
     37f986522545 Extracting 1 s
     c34808af6b3e Pull complete
     4ef363dc51e7 Downloading [===============>                                   ]  22.02MB/68.83MB
     37f986522545 Extracting 1 s
     4ef363dc51e7 Downloading [================>                                  ]  23.07MB/68.83MB
     37f986522545 Extracting 1 s
     4ef363dc51e7 Downloading [=================>                                 ]  24.12MB/68.83MB
     37f986522545 Pull complete
     3e968919cb92 Pull complete
     4ef363dc51e7 Downloading [==================>                                ]  25.17MB/68.83MB
     e7908f300846 Pull complete
     4ef363dc51e7 Downloading [===================>                               ]  26.21MB/68.83MB
     4ef363dc51e7 Downloading [===================>                               ]  27.26MB/68.83MB
     4ef363dc51e7 Downloading [=====================>                             ]  29.36MB/68.83MB
     4ef363dc51e7 Downloading [======================>                            ]  30.41MB/68.83MB
     4ef363dc51e7 Downloading [======================>                            ]  31.46MB/68.83MB
     4ef363dc51e7 Downloading [=======================>                           ]  32.51MB/68.83MB
     4ef363dc51e7 Downloading [========================>                          ]  33.55MB/68.83MB
     4ef363dc51e7 Downloading [=========================>                         ]   34.6MB/68.83MB
     4ef363dc51e7 Downloading [=========================>                         ]  35.65MB/68.83MB
     4ef363dc51e7 Downloading [==========================>                        ]   36.7MB/68.83MB
     4ef363dc51e7 Downloading [============================>                      ]   38.8MB/68.83MB
     4ef363dc51e7 Downloading [============================>                      ]  39.85MB/68.83MB
     4ef363dc51e7 Downloading [=============================>                     ]  40.89MB/68.83MB
     4ef363dc51e7 Downloading [==============================>                    ]  41.94MB/68.83MB
     4ef363dc51e7 Downloading [===============================>                   ]  42.99MB/68.83MB
     4ef363dc51e7 Downloading [===============================>                   ]  44.04MB/68.83MB
     4ef363dc51e7 Downloading [================================>                  ]  45.09MB/68.83MB
     4ef363dc51e7 Downloading [=================================>                 ]  46.14MB/68.83MB
     4ef363dc51e7 Downloading [==================================>                ]  47.19MB/68.83MB
     4ef363dc51e7 Downloading [===================================>               ]  49.28MB/68.83MB
     4ef363dc51e7 Downloading [====================================>              ]  50.33MB/68.83MB
     4ef363dc51e7 Downloading [=====================================>             ]  51.38MB/68.83MB
     4ef363dc51e7 Downloading [======================================>            ]  52.43MB/68.83MB
     4ef363dc51e7 Downloading [======================================>            ]  53.48MB/68.83MB
     4ef363dc51e7 Downloading [=======================================>           ]  54.53MB/68.83MB
     4ef363dc51e7 Downloading [========================================>          ]  55.57MB/68.83MB
     4ef363dc51e7 Downloading [=========================================>         ]  56.62MB/68.83MB
     4ef363dc51e7 Downloading [==========================================>        ]  58.72MB/68.83MB
     4ef363dc51e7 Downloading [===========================================>       ]  59.77MB/68.83MB
     4ef363dc51e7 Downloading [============================================>      ]  60.82MB/68.83MB
     4ef363dc51e7 Downloading [============================================>      ]  61.87MB/68.83MB
     4ef363dc51e7 Downloading [=============================================>     ]  62.91MB/68.83MB
     4ef363dc51e7 Downloading [==============================================>    ]  63.96MB/68.83MB
     4ef363dc51e7 Downloading [===============================================>   ]  65.01MB/68.83MB
     4ef363dc51e7 Downloading [===============================================>   ]  66.06MB/68.83MB
     4ef363dc51e7 Downloading [================================================>  ]  67.11MB/68.83MB
     4ef363dc51e7 Downloading [==================================================>]  68.83MB/68.83MB
     4ef363dc51e7 Downloading [==================================================>]  68.83MB/68.83MB
     4ef363dc51e7 Download complete
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 1 s
     4ef363dc51e7 Extracting 2 s
     4ef363dc51e7 Extracting 2 s
     4ef363dc51e7 Extracting 2 s
     59013e7d205a Pull complete
     d626f8bd756e Pull complete
     4ef363dc51e7 Pull complete
     db Pulled
     Network partkeepr-ng_default  Creating
     Network partkeepr-ng_default  Created
     Volume "partkeepr-ng_database"  Creating
     Volume "partkeepr-ng_database"  Created
     Container partkeepr-ng-db-1  Creating
     Container partkeepr-ng-db-1  Created
     Container partkeepr-ng-db-1  Starting
     Container partkeepr-ng-db-1  Started
    ✓ • 13.88s

## Activity

    $ docker compose -f local-install/compose.yaml exec -T db sh -c 'exec mariadb -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"' < /tmp/partkeepr-prod.sql
    ✓ • 788ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import subprocess
        base=['docker','compose','-f','local-install/compose.yaml','exec','-T','db','sh','-c','exec mariadb -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"']
        for path in sorted(Path('scripts/migrations').glob('[0-9]*.sql')):
         print('Applying',path.name,flush=True)
         with path.open('rb') as sql:
          result=subprocess.run(base,stdin=sql,capture_output=True)
         print(result.stdout.decode(),end='')
         if result.returncode:
          print(result.stderr.decode());raise SystemExit(result.returncode)
        Path('local-install/migrations-applied.txt').write_text('\n'.join(p.name for p in sorted(Path('scripts/migrations').glob('[0-9]*.sql')))+'\n')
        PY
    Applying 0001-attribution-backfill.sql
    system_user_id    stock_null_after    project_null_after
    2    0    0
    Applying 0002-multi-location-parts.sql
    Applying 0003-stock-entry-order-tracking.sql
    backfilled_rows
    0
    Applying 0004-container-rows-backfill.sql
    containers_after_backfill    total_qty_in_containers
    577    19298
    Applying 0005-partunit-cleanup.sql
    id    name    shortName    is_default
    1    Pieces    pcs    1
    3    Centimetres    cm    0
    7    Grams    g    0
    6    Kilograms    kg    0
    4    Litres    L    0
    2    Metres    m    0
    5    Millilitres    mL    0
    8    Pairs    pr    0
    9    Sets    set    0
    Applying 0006-stock-entry-container-tracking.sql
    ✓ • 3.09s

## Activity

    $ python3 - <<'PY'
        import subprocess,json
        from pathlib import Path
        query="SELECT 'Part',COUNT(*) FROM Part UNION ALL SELECT 'StockEntry',COUNT(*) FROM StockEntry UNION ALL SELECT 'Project',COUNT(*) FROM Project;"
        base=['docker','compose','-f','local-install/compose.yaml','exec','-T','db','sh','-c','exec mariadb -N -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"']
        def sql(q):
         r=subprocess.run(base,input=q,text=True,capture_output=True,check=True);return r.stdout
        print(sql(query))
        missing=[];counts={}
        for table in ['PartAttachment','FootprintAttachment','ProjectAttachment','FootprintImage','ManufacturerICLogo','StorageLocationImage']:
         rows=sql(f'SELECT id,filename,COALESCE(extension,\'\') FROM {table};').splitlines()
         counts[table]=len(rows)
         directory=Path('local-install/storage')/table
         for row in rows:
          id,name,ext=row.split('\t')
          matches=list(directory.glob(name+'.*')) if directory.exists() else []
          if not (directory/name).exists() and not any('.thumb.' not in p.name for p in matches): missing.append({'kind':table,'id':id,'filename':name,'extension':ext})
        report={'database_file_counts':counts,'missing_files':missing}
        Path('local-install/file-audit.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
        PY
    Part    669
    StockEntry    605
    Project    0
    
    {
      "database_file_counts": {
        "PartAttachment": 254,
        "FootprintAttachment": 1,
        "ProjectAttachment": 0,
        "FootprintImage": 37,
        "ManufacturerICLogo": 346,
        "StorageLocationImage": 65
      },
      "missing_files": []
    }
    ✓ • 3.27s

## Activity

    $ docker compose -f local-install/compose.yaml exec -T db sh -c 'exec mariadb -N -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE" -e "SELECT id,filename,extension FROM StorageLocationImage LIMIT 8; SELECT COUNT(DISTINCT filename),SUM(filename = CHAR(0)),SUM(filename = '\'' '\'') FROM PartAttachment;"'; id -u; id -g; sed -n '1,200p' backend/src/session.rs; sed -n '1,130p' backend/src/handlers/auth.rs
    1    83fa0050-f3aa-11f0-9004-66c063297014    jpeg
    2    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    3    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    4    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    5    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    6    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    7    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    8    ff4c7a42-f785-11f0-97a3-1f201b3bb2a5
    232    0    0
    1000
    1000
    sed: can't read backend/src/session.rs: No such file or directory
    // SPDX-License-Identifier: GPL-3.0-or-later
    
    //! Slice 10a: authentication.
    //!
    //! Login verifies against the legacy `FOSUser` table's password hash
    //! (FOSUserBundle's default: sha512 over `password{salt}` iterated 5000
    //! times, base64-encoded), then looks up the matching `PartKeeprUser`
    //! row by username — the legacy app-table that all the `user_id` FK
    //! columns point at. The session is an HMAC-signed payload stored in
    //! a `partkeepr_session` cookie; no DB session table.
    //!
    //! The middleware in `auth_middleware` runs ahead of every other
    //! handler. Endpoints in `PUBLIC_PATHS` (`/health`, `/api/login`)
    //! pass through unauthenticated; everything else returns 401 unless
    //! the cookie carries a valid, unexpired session.
    
    use axum::{
        extract::{Request, State},
        http::StatusCode,
        middleware::Next,
        response::{IntoResponse, Response},
        Json,
    };
    use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
    use base64::Engine;
    use chrono::{Duration, Utc};
    use hmac::{Hmac, Mac};
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256, Sha512};
    use sqlx::MySqlPool;
    use subtle::ConstantTimeEq;
    use time::Duration as TimeDuration;
    
    use crate::error::AppError;
    
    /// Cookie name. Stable; renaming this invalidates all sessions.
    pub const SESSION_COOKIE: &str = "partkeepr_session";
    
    /// Default session lifetime. We set the cookie's Max-Age to match.
    const SESSION_TTL_HOURS: i64 = 12;
    
    /// Paths that bypass auth. Order doesn't matter; we exact-match.
    const PUBLIC_PATHS: &[&str] = &["/health", "/api/login"];
    
    // ---------------------------------------------------------------------------
    //  Session payload
    // ---------------------------------------------------------------------------
    
    /// Decoded session, attached to authenticated requests via the
    /// auth middleware. Handlers extract via `Extension<CurrentUser>`.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CurrentUser {
        /// `PartKeeprUser.id` — what the FK columns reference.
        pub user_id: i32,
        /// `FOSUser.id` — needed for any `last_login` updates etc.
        pub fos_user_id: i32,
        pub username: String,
        pub is_admin: bool,
        /// Unix-seconds expiry. Re-checked at middleware time.
        pub exp: i64,
    }
    
    impl CurrentUser {
        fn is_expired(&self) -> bool {
            Utc::now().timestamp() >= self.exp
        }
    }
    
    // ---------------------------------------------------------------------------
    //  HMAC-signed cookie payload
    // ---------------------------------------------------------------------------
    
    /// Wraps the session-signing secret loaded once at startup. Cloned
    /// freely (it's an `Arc`-sized handle internally — the secret is a
    /// short byte string).
    #[derive(Clone)]
    pub struct SessionSigner {
        secret: Vec<u8>,
    }
    
    impl SessionSigner {
        /// Load from env. Required — the server refuses to start without
        /// `PARTKEEPR_SESSION_SECRET` set, so a random secret can't
        /// silently invalidate everyone's sessions on every restart.
        pub fn from_env() -> anyhow::Result<Self> {
            let s = std::env::var("PARTKEEPR_SESSION_SECRET").map_err(|_| {
                anyhow::anyhow!(
                    "PARTKEEPR_SESSION_SECRET must be set (32+ random bytes; \
                     generate with `openssl rand -base64 48`)"
                )
            })?;
            if s.len() < 16 {
                anyhow::bail!("PARTKEEPR_SESSION_SECRET must be at least 16 chars");
            }
            Ok(Self { secret: s.into_bytes() })
        }
    
        /// Encode `payload` as `base64url(json) + "." + base64url(hmac)`.
        fn encode(&self, payload: &CurrentUser) -> Result<String, AppError> {
            let json = serde_json::to_string(payload)
                .map_err(|_| AppError::BadRequest("session encode failed"))?;
            let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());
            let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(&self.secret)
                .map_err(|_| AppError::BadRequest("session sign failed"))?;
            mac.update(payload_b64.as_bytes());
            let sig = mac.finalize().into_bytes();
            let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig);
            Ok(format!("{payload_b64}.{sig_b64}"))
        }
    
        /// Decode + verify a cookie value. Returns the payload on success,
        /// `None` if signature mismatch / parse error / expired.
        fn decode(&self, raw: &str) -> Option<CurrentUser> {
            let (payload_b64, sig_b64) = raw.split_once('.')?;
            // Recompute signature over the payload portion and compare in
            // constant time. Mismatch → reject.
            let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(&self.secret).ok()?;
            mac.update(payload_b64.as_bytes());
            let expected_sig = mac.finalize().into_bytes();
            let expected_b64 =
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(expected_sig);
            if !bool::from(expected_b64.as_bytes().ct_eq(sig_b64.as_bytes())) {
                return None;
            }
            let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(payload_b64)
                .ok()?;
            let payload: CurrentUser = serde_json::from_slice(&json_bytes).ok()?;
            if payload.is_expired() {
                return None;
    ✓ • 238ms

## Activity

    $ rg -n '\.route\(' backend/src/handlers/mod.rs | head -60; sed -n '1,120p' backend/src/startup.rs; df -h .
    40:        .route("/api/login", post(auth::login))
    41:        .route("/api/logout", post(auth::logout))
    42:        .route("/api/me", get(auth::me))
    43:        .route("/api/version", get(version::version))
    44:        .route("/api/part_categories/tree", get(parts::category_tree))
    45:        .route(
    49:        .route(
    53:        .route(
    57:        .route("/api/parts", get(parts::list).post(parts::create))
    61:        .route("/api/parts/by_distributor_sku", get(parts::by_distributor_sku))
    62:        .route(
    66:        .route("/api/parts/:id/stock-entries", post(stock::create_stock_entry))
    67:        .route("/api/parts/:id/stock-receipts", get(stock::list_part_receipts))
    69:        .route("/api/storage_tree", get(storage_locations::tree))
    70:        .route(
    74:        .route(
    79:        .route(
    83:        .route(
    87:        .route(
    92:        .route(
    97:        .route("/api/footprint_tree", get(footprints::tree))
    98:        .route(
    102:        .route(
    106:        .route(
    110:        .route(
    114:        .route(
    118:        .route("/api/footprints/:id/move", patch(footprints::fp_move))
    120:        .route(
    124:        .route(
    128:        .route(
    132:        .route(
    136:        .route(
    141:        .route("/api/parts/:id/attachments/by-url", post(attachments::fetch_part))
    142:        .route("/api/footprints/:id/images/by-url", post(attachments::fetch_footprint_image))
    143:        .route("/api/footprints/:id/attachments/by-url", post(attachments::fetch_footprint_attachment))
    144:        .route("/api/manufacturers/:id/logos/by-url", post(attachments::fetch_manufacturer_logo))
    145:        .route("/api/storage_locations/:id/images/by-url", post(attachments::fetch_storage_location_image))
    146:        .route("/files/:kind/:id", get(attachments::download))
    147:        .route("/files/:kind/:id/thumb", get(attachments::download_thumb))
    148:        .route(
    153:        .route(
    157:        .route(
    161:        .route("/api/manufacturers/:id/merge_into", post(lookups::mfg_merge))
    162:        .route(
    166:        .route(
    170:        .route("/api/distributors/:id/merge_into", post(lookups::dist_merge))
    171:        .route(
    175:        .route(
    179:        .route("/api/part_measurement_units/:id/merge_into", post(lookups::part_unit_merge))
    180:        .route(
    184:        .route(
    188:        .route("/api/units/:id/merge_into", post(lookups::unit_merge))
    189:        .route(
    193:        .route(
    197:        .route("/api/si_prefixes/:id/merge_into", post(lookups::siprefix_merge))
    199:        .route("/api/projects", get(projects::list).post(projects::create))
    200:        .route(
    206:        .route("/api/projects/:id/parts", post(projects::add_part))
    207:        .route(
    212:        .route(
    // SPDX-License-Identifier: GPL-3.0-or-later
    
    //! Startup invariant checks. Runs once after the DB pool is built,
    //! before the server starts accepting requests. The single rule:
    //! everything in here must be idempotent — repeated runs on a
    //! healthy DB are a no-op.
    //!
    //! Today this only ensures the operator's "(NOWHERE)" catch-all
    //! StorageLocation exists. New invariants (schema checks, missing
    //! seed rows, etc.) belong here too.
    
    use anyhow::Context;
    use sqlx::MySqlPool;
    
    /// The convention bin name for "not-yet-placed" parts. Created if
    /// missing on startup; can't be deleted via the API. The literal
    /// string is also matched by the frontend's defaultStorageLocationId
    /// helper (case-insensitive regex against `NOWHERE`), so renaming
    /// this here would break that lookup.
    pub const NOWHERE_LOCATION_NAME: &str = "(NOWHERE)";
    
    /// Run all startup invariant checks. Bails if any check fails so
    /// the operator sees the problem at boot rather than at first
    /// request.
    pub async fn ensure_invariants(pool: &MySqlPool) -> anyhow::Result<()> {
        ensure_nowhere_storage_location(pool).await?;
        Ok(())
    }
    
    /// Ensure a StorageLocation named `(NOWHERE)` exists. Used as the
    /// system-wide default for parts whose physical bin hasn't been
    /// chosen yet. Idempotent.
    async fn ensure_nowhere_storage_location(pool: &MySqlPool) -> anyhow::Result<()> {
        let existing: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM StorageLocation WHERE name = ? LIMIT 1",
        )
        .bind(NOWHERE_LOCATION_NAME)
        .fetch_optional(pool)
        .await
        .context("checking for (NOWHERE) StorageLocation")?;
    
        if existing.is_some() {
            tracing::debug!(name = %NOWHERE_LOCATION_NAME, "convention bin exists");
            return Ok(());
        }
    
        // Pick a category to attach the new bin to. StorageLocationCategory
        // is a nested-set tree; the root is the only guaranteed category.
        // Find its id; bail if even that's missing (would mean an
        // uninitialized DB).
        let cat_id: i32 = sqlx::query_scalar(
            "SELECT id FROM StorageLocationCategory \
             WHERE parent_id IS NULL OR parent_id = 0 \
             ORDER BY id LIMIT 1",
        )
        .fetch_one(pool)
        .await
        .context("looking up root StorageLocationCategory")?;
    
        sqlx::query(
            "INSERT INTO StorageLocation (name, category_id) VALUES (?, ?)",
        )
        .bind(NOWHERE_LOCATION_NAME)
        .bind(cat_id)
        .execute(pool)
        .await
        .context("creating (NOWHERE) StorageLocation")?;
    
        tracing::info!(name = %NOWHERE_LOCATION_NAME, "created convention bin");
        Ok(())
    }
    Filesystem      Size  Used Avail Use% Mounted on
    /dev/sda2       937G  113G  777G  13% /home/alan/partkeepr-ng
    ✓ • 159ms

## Assistant

The database restore and all six migrations succeeded: 669 parts and 605 stock-history entries are present. Every attachment and image record has a matching file in the archive, so no additional extraction is needed for those files. Some records share the same file, which explains why the database has more records than physical files.

The application is still compiling. Next I’ll start it and check the API, file downloads, and thumbnail generation.

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('local-install')
        (p/'.dockerignore').write_text('*\n!Dockerfile\n!bin/\n!bin/partkeepr-api\n')
        (p/'README.md').write_text('''# Local PartKeepr NG installation
        
        URL: http://localhost:8082/ (bound to localhost).
        Use existing PartKeepr credentials. Original containers are unchanged.
        
        From /home/alan/partkeepr-ng:
        
            docker compose -f local-install/compose.yaml ps
            docker compose -f local-install/compose.yaml logs --tail=100 app
            docker compose -f local-install/compose.yaml stop
            docker compose -f local-install/compose.yaml up -d
        
        Docker restart policy: unless-stopped. Database uses Docker volume
        partkeepr-ng_database. Files are in local-install/storage.
        Credentials and session secret are in local-install/database.env and app.env
        (mode 600); keep these private. All local-install contents are gitignored.
        Do not run compose down --volumes: that deletes the imported database.
        
        Source inputs:
        - /tmp/partkeepr-prod.sql
        - /home/alan/partkeepr-files.tar.gz
        
        All six migrations were applied once; see migrations-applied.txt.
        File reconciliation is recorded in file-audit.json. Legacy image directories
        were mapped to FootprintImage, ManufacturerICLogo, StorageLocationImage.
        Transient archive files were skipped. Original archive/dump remain unchanged.
        
        Back up both the database and storage while the app is stopped:
        
            docker compose -f local-install/compose.yaml stop app
            docker compose -f local-install/compose.yaml exec -T db sh -c 'exec mariadb-dump --single-transaction -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"' > /YOUR/BACKUP/PATH/partkeepr-ng.sql
            tar -czf /YOUR/BACKUP/PATH/partkeepr-ng-files.tar.gz -C local-install storage
            docker compose -f local-install/compose.yaml start app
        
        After source updates, build and refresh the runtime image:
        
            cargo +stable build --locked --release --bin partkeepr-api
            cp target/release/partkeepr-api local-install/bin/partkeepr-api
            docker compose -f local-install/compose.yaml up -d --build app
        
        Review new migration requirements before updating. Do not rerun the original
        migration set blindly; migration 0006 is not idempotent.
        ''')
        print('Wrote local operating and backup instructions.')
        PY
    Wrote local operating and backup instructions.
    ✓ • 14ms

## Activity

    $ sed -n '185,315p' backend/src/handlers/auth.rs; sed -n '315,430p' backend/src/handlers/attachments.rs; sed -n '35,80p' backend/src/handlers/mod.rs; git status --short
        pub username: String,
        pub is_admin: bool,
    }
    
    /// `POST /api/login` — verify credentials, set the session cookie.
    ///
    /// Looks up the FOSUser by `username_canonical` (lowercased), verifies
    /// the legacy hash, then matches a PartKeeprUser row by username for
    /// the FK attribution. Either lookup failing → 401 (vague on purpose
    /// to avoid username-enumeration via differential errors).
    pub async fn login(
        State(pool): State<MySqlPool>,
        State(signer): State<SessionSigner>,
        jar: CookieJar,
        Json(req): Json<LoginRequest>,
    ) -> Result<Response, AppError> {
        let username = req.username.trim();
        if username.is_empty() || req.password.is_empty() {
            return Err(AppError::BadRequest(
                "username and password are required",
            ));
        }
        let canonical = username.to_lowercase();
    
        #[derive(sqlx::FromRow)]
        struct FosRow {
            id: i32,
            username: String,
            password: String,
            salt: String,
            enabled: bool,
            locked: bool,
        }
        let fos: Option<FosRow> = sqlx::query_as(
            "SELECT id, username, password, salt, enabled, locked \
             FROM FOSUser WHERE username_canonical = ?",
        )
        .bind(&canonical)
        .fetch_optional(&pool)
        .await?;
        // Constant-ish timing: always run the verify even when the user
        // doesn't exist, to dodge user-existence side channels via response
        // latency. (Not a hard guarantee, but stops the obvious case.)
        let dummy = FosRow {
            id: 0,
            username: String::new(),
            password: "X".repeat(88),
            salt: "X".repeat(31),
            enabled: false,
            locked: true,
        };
        let row = fos.as_ref().unwrap_or(&dummy);
        let pw_ok = verify_fos_password(&req.password, &row.salt, &row.password);
    
        if fos.is_none() || !pw_ok || !row.enabled || row.locked {
            return Err(AppError::Unauthorized(
                "username or password not recognised",
            ));
        }
        let fos = fos.unwrap();
    
        // Find the matching PartKeeprUser. Lookup by username (the
        // FOSUser/PartKeeprUser tables are linked implicitly; not every
        // FOSUser has a PartKeeprUser row — the schema has at least one
        // such orphan today). Treat missing PartKeeprUser as 401 too —
        // we can't attribute writes to them.
        let pk: Option<(i32, bool)> = sqlx::query_as(
            "SELECT id, admin FROM PartKeeprUser \
             WHERE username = ? AND active = 1 \
             LIMIT 1",
        )
        .bind(&fos.username)
        .fetch_optional(&pool)
        .await?;
        let (pk_id, is_admin) = pk.ok_or(AppError::Unauthorized(
            "no PartKeeprUser linked to this account",
        ))?;
    
        // Touch last_login so the legacy data stays useful for ops insight.
        let _ = sqlx::query("UPDATE FOSUser SET last_login = NOW() WHERE id = ?")
            .bind(fos.id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("UPDATE PartKeeprUser SET lastSeen = NOW() WHERE id = ?")
            .bind(pk_id)
            .execute(&pool)
            .await;
    
        let exp = (Utc::now() + Duration::hours(SESSION_TTL_HOURS)).timestamp();
        let session = CurrentUser {
            user_id: pk_id,
            fos_user_id: fos.id,
            username: fos.username.clone(),
            is_admin,
            exp,
        };
        let cookie_value = signer.encode(&session)?;
    
        // PARTKEEPR_SECURE_COOKIES=true forces the Secure attribute on
        // the session cookie. Set this in production (behind an https
        // reverse proxy); leave unset for local dev over plain http.
        let secure = std::env::var("PARTKEEPR_SECURE_COOKIES")
            .ok()
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false);
    
        let cookie = Cookie::build((SESSION_COOKIE, cookie_value))
            .path("/")
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Strict)
            .max_age(TimeDuration::hours(SESSION_TTL_HOURS))
            .build();
    
        let body = LoginResponse {
            user: CurrentUserView {
                user_id: pk_id,
                username: session.username.clone(),
                is_admin,
            },
        };
        Ok((jar.add(cookie), Json(body)).into_response())
    }
    
    // ---------------------------------------------------------------------------
    //  Logout / me
    // ---------------------------------------------------------------------------
    
    /// `POST /api/logout` — drop the session cookie. Idempotent (works
    /// even when not currently authenticated).
    pub async fn logout(jar: CookieJar) -> Response {
            ))));
        }
    
        // Pre-flight CL check: if the header says it's bigger than our cap,
        // bail before we read a single byte.
        if let Some(cl) = resp.content_length() {
            if cl as usize > URL_FETCH_MAX_BYTES {
                return Err(AppError::BadRequest(leak(format!(
                    "remote file too large: {cl} bytes (cap {URL_FETCH_MAX_BYTES})"
                ))));
            }
        }
    
        // Pull a few bits out of the response BEFORE moving it into the
        // bytes_stream() consumer: content-type, content-disposition, the
        // resolved final URL (after redirects).
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());
        let cd_filename = resp.headers().get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_content_disposition_filename);
        let final_url = resp.url().clone();
    
        // Stream the body, abort if it exceeds cap.
        let mut buf = BytesMut::with_capacity(64 * 1024);
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| AppError::BadRequest(leak(format!("read body failed: {e}"))))?;
            if buf.len() + chunk.len() > URL_FETCH_MAX_BYTES {
                return Err(AppError::BadRequest(leak(format!(
                    "remote file exceeds cap of {URL_FETCH_MAX_BYTES} bytes"
                ))));
            }
            buf.extend_from_slice(&chunk);
        }
        let bytes = buf.freeze();
        if bytes.is_empty() {
            return Err(AppError::BadRequest("remote returned empty body"));
        }
    
        // Filename selection: explicit override > Content-Disposition >
        // last URL path segment > generic.
        let original_filename = req.filename
            .filter(|s| !s.trim().is_empty())
            .or(cd_filename)
            .or_else(|| {
                final_url.path_segments()
                    .and_then(|segs| segs.filter(|s| !s.is_empty()).last())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "url-fetch".to_string());
    
        let saved_file = match storage::save_upload(
            store,
            kind.name(),
            UploadInput {
                bytes,
                original_filename: Some(original_filename.clone()),
                content_type,
            },
        ).await {
            Ok(s) => s,
            Err(SaveError::Infected(sig)) => {
                return Err(AppError::BadRequest(leak(format!(
                    "fetch rejected — virus scanner flagged: {sig}"
                ))));
            }
            Err(SaveError::Io(e)) => {
                return Err(AppError::BadRequest(leak(format!(
                    "could not save fetched file: {e}"
                ))));
            }
            Err(SaveError::Other(e)) => {
                tracing::error!(error = %e, "url-fetch save failed");
                return Err(AppError::BadRequest("fetch failed"));
            }
        };
    
        let now = chrono::Utc::now().naive_utc();
        let new_id = if kind.has_is_image_column() {
            sqlx::query(&format!(
                "INSERT INTO {tbl} \
                    ({fk}, type, filename, originalname, mimetype, size, \
                     description, extension, isImage, created) \
                 VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?, ?)",
                tbl = kind.table(), fk = kind.parent_fk()
            ))
            .bind(parent_id).bind(kind.name())
            .bind(&saved_file.uuid).bind(&saved_file.original_filename)
            .bind(&saved_file.mimetype).bind(saved_file.size)
            .bind(&saved_file.extension).bind(saved_file.is_image)
            .bind(now).execute(pool).await?.last_insert_id() as i32
        } else {
            sqlx::query(&format!(
                "INSERT INTO {tbl} \
                    ({fk}, type, filename, originalname, mimetype, size, \
                     description, extension, created) \
                 VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?)",
                tbl = kind.table(), fk = kind.parent_fk()
            ))
            .bind(parent_id).bind(kind.name())
            .bind(&saved_file.uuid).bind(&saved_file.original_filename)
            .bind(&saved_file.mimetype).bind(saved_file.size)
            .bind(&saved_file.extension)
            .bind(now).execute(pool).await?.last_insert_id() as i32
        };
    
        Ok(Json(vec![AttachmentRow {
            id: new_id,
            filename: saved_file.uuid,
            originalname: Some(saved_file.original_filename),
            mimetype: saved_file.mimetype,
            size: saved_file.size,
            description: None,
    /// codegen routes from `models::routes()`. Routes here take precedence by
    /// not being in codegen output (see SKIP_HANDLER_CLASSES in the codegen).
    pub fn routes() -> Router<AppState> {
        Router::new()
            // Slice 10a — auth.
            .route("/api/login", post(auth::login))
            .route("/api/logout", post(auth::logout))
            .route("/api/me", get(auth::me))
            .route("/api/version", get(version::version))
            .route("/api/part_categories/tree", get(parts::category_tree))
            .route(
                "/api/part_categories",
                get(categories::list_flat).post(categories::create),
            )
            .route(
                "/api/part_categories/:id",
                axum::routing::put(categories::update).delete(categories::delete),
            )
            .route(
                "/api/part_categories/:id/move",
                patch(categories::move_node),
            )
            .route("/api/parts", get(parts::list).post(parts::create))
            // Slice 13b: resolve a distributor SKU → matched part +
            // distributor identity, used by the barcode-scan dispatch
            // for correct attribution on the StockEntry.
            .route("/api/parts/by_distributor_sku", get(parts::by_distributor_sku))
            .route(
                "/api/parts/:id",
                get(parts::detail).put(parts::update).delete(parts::delete),
            )
            .route("/api/parts/:id/stock-entries", post(stock::create_stock_entry))
            .route("/api/parts/:id/stock-receipts", get(stock::list_part_receipts))
            // Slice 5c-2: storage locations.
            .route("/api/storage_tree", get(storage_locations::tree))
            .route(
                "/api/storage_location_categories",
                get(storage_locations::cat_list_flat).post(storage_locations::cat_create),
            )
            .route(
                "/api/storage_location_categories/:id",
                axum::routing::put(storage_locations::cat_update)
                    .delete(storage_locations::cat_delete),
            )
            .route(
                "/api/storage_location_categories/:id/move",
    ✓ • 0ms

## Activity

    $ cat > local-install/verify.py <<'PY'
        import base64, hashlib, hmac, json, subprocess, time, urllib.request, urllib.error
        from pathlib import Path
        base=['docker','compose','-f','local-install/compose.yaml','exec','-T','db','sh','-c','exec mariadb -N -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"']
        def sql(q):
            return subprocess.run(base,input=q,text=True,capture_output=True,check=True).stdout
        row=sql('SELECT p.id,f.id,f.username,p.admin FROM PartKeeprUser p JOIN FOSUser f ON f.username=p.username WHERE p.active=1 AND f.enabled=1 AND f.locked=0 LIMIT 1;').strip().split('\t')
        assert len(row)==4, 'No active matching legacy account'
        env=dict(line.split('=',1) for line in Path('local-install/app.env').read_text().splitlines())
        payload={'user_id':int(row[0]),'fos_user_id':int(row[1]),'username':row[2],'is_admin':row[3]=='1','exp':int(time.time())+120}
        def b64(data): return base64.urlsafe_b64encode(data).rstrip(b'=')
        encoded=b64(json.dumps(payload,separators=(',',':')).encode())
        sig=b64(hmac.new(env['PARTKEEPR_SESSION_SECRET'].encode(),encoded,hashlib.sha256).digest())
        cookie='partkeepr_session='+(encoded+b'.'+sig).decode()
        results=[]
        def get(path,auth=True):
            req=urllib.request.Request('http://127.0.0.1:8082'+path,headers={'Cookie':cookie} if auth else {})
            with urllib.request.urlopen(req,timeout=30) as response:
                body=response.read()
                assert response.status==200
                results.append({'path':path,'status':response.status,'bytes':len(body)})
                return body
        assert get('/health',False)==b'ok'
        get('/',False)
        try:
            get('/api/me',False)
            raise AssertionError('Unauthenticated request accepted')
        except urllib.error.HTTPError as e:
            assert e.code==401
            results.append({'path':'/api/me (unauthenticated)','status':401})
        for path in ['/api/me','/api/version','/api/parts','/api/part_categories/tree','/api/storage_tree','/api/footprint_tree','/api/projects']:
            json.loads(get(path))
        part_id=sql('SELECT id FROM Part ORDER BY id LIMIT 1;').strip()
        json.loads(get('/api/parts/'+part_id))
        for kind in ['PartAttachment','FootprintAttachment','ProjectAttachment','FootprintImage','ManufacturerICLogo','StorageLocationImage']:
            id=sql(f'SELECT id FROM {kind} ORDER BY id LIMIT 1;').strip()
            if not id: continue
            get('/files/'+kind+'/'+id)
            if kind in ['FootprintImage','ManufacturerICLogo','StorageLocationImage']:
                thumb=get('/files/'+kind+'/'+id+'/thumb')
                assert thumb.startswith(b'\xff\xd8'), 'Expected JPEG thumbnail'
        Path('local-install/verification.json').write_text(json.dumps({'method':'Read-only requests using a short-lived session signed with installation secret; original password login not tested.','results':results},indent=2)+'\n')
        for result in results: print(result)
        PY
    ✓ • 0ms

## Activity

    $ docker compose -f local-install/compose.yaml ps; docker inspect --format '{{.HostConfig.RestartPolicy.Name}}' partkeepr-ng-db-1; ps -eo comm,etime,%cpu --sort=-%cpu | head -8
    NAME                IMAGE           COMMAND                  SERVICE   CREATED         STATUS                   PORTS
    partkeepr-ng-db-1   mariadb:10.11   "docker-entrypoint.s…"   db        2 minutes ago   Up 2 minutes (healthy)   3306/tcp
    unless-stopped
    COMMAND             ELAPSED %CPU
    rustc                 00:10  103
    zsh                   00:00 17.6
    packagekitd           05:01  7.4
    cargo                 01:59  6.4
    codex                 06:57  2.7
    gnome-shell     42-13:55:01  1.6
    invoke          24-21:21:44  1.5
    ✓ • 45ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import json
        p=Path('local-install/file-audit.json');report=json.loads(p.read_text())
        report['physical_file_counts']={directory.name:sum(1 for f in directory.iterdir() if f.is_file() and '.thumb.' not in f.name) for directory in Path('local-install/storage').iterdir() if directory.is_dir()}
        p.write_text(json.dumps(report,indent=2)+'\n')
        print('File audit saved with database and physical-file counts.')
        PY
    File audit saved with database and physical-file counts.
    ✓ • 848ms

## Activity

    $ cargo +stable build --locked --release --bin partkeepr-api
        Updating crates.io index
     Downloading crates ...
      Downloaded sqlx-macros-core v0.8.6
      Downloaded sqlx-macros v0.8.6
      Downloaded stable_deref_trait v1.2.1
      Downloaded sync_wrapper v1.0.2
      Downloaded tap v1.0.1
      Downloaded parking_lot_core v0.9.12
      Downloaded spki v0.7.3
      Downloaded adler2 v2.0.1
      Downloaded try-lock v0.2.5
      Downloaded byteorder-lite v0.1.0
      Downloaded yoke-derive v0.8.2
      Downloaded time-core v0.1.8
      Downloaded untrusted v0.9.0
      Downloaded block-buffer v0.10.4
      Downloaded want v0.3.1
      Downloaded atoi v2.0.0
      Downloaded bytecheck_derive v0.6.12
      Downloaded zerofrom-derive v0.1.7
      Downloaded tinyvec_macros v0.1.1
      Downloaded potential_utf v0.1.5
      Downloaded tokio-macros v2.7.0
      Downloaded tower-layer v0.3.3
      Downloaded atomic-waker v1.1.2
      Downloaded subtle v2.6.1
      Downloaded stringprep v0.1.5
      Downloaded synstructure v0.13.2
      Downloaded ptr_meta_derive v0.1.4
      Downloaded tower-service v0.3.3
      Downloaded zerofrom v0.1.7
      Downloaded powerfmt v0.2.0
      Downloaded radium v0.7.0
      Downloaded quick-error v2.0.1
      Downloaded bytecheck v0.6.12
      Downloaded utf8_iter v1.0.4
      Downloaded ptr_meta v0.1.4
      Downloaded version_check v0.9.5
      Downloaded pem-rfc7468 v0.7.0
      Downloaded webpki-roots v0.26.11
      Downloaded crossbeam-queue v0.3.12
      Downloaded num-iter v0.1.45
      Downloaded thiserror v2.0.18
      Downloaded thiserror-impl v2.0.18
      Downloaded toml_datetime v1.1.1+spec-1.1.0
      Downloaded percent-encoding v2.3.2
      Downloaded writeable v0.6.3
      Downloaded axum-core v0.4.5
      Downloaded proc-macro-crate v3.5.0
      Downloaded async-trait v0.1.89
      Downloaded tracing-log v0.2.0
      Downloaded wyz v0.5.1
      Downloaded cfg-if v1.0.4
      Downloaded rustc-hash v2.1.2
      Downloaded futures-sink v0.3.32
      Downloaded errno v0.3.14
      Downloaded home v0.5.12
      Downloaded pkcs8 v0.10.2
      Downloaded http-range-header v0.4.2
      Downloaded thread_local v1.1.9
      Downloaded pin-project-lite v0.2.17
      Downloaded rand_core v0.9.5
      Downloaded rkyv_derive v0.7.46
      Downloaded byteorder v1.5.0
      Downloaded arrayvec v0.7.6
      Downloaded zerovec-derive v0.11.3
      Downloaded pkcs1 v0.7.5
      Downloaded ppv-lite86 v0.2.21
      Downloaded heck v0.5.0
      Downloaded tinystr v0.8.3
      Downloaded autocfg v1.5.0
      Downloaded zmij v1.0.21
      Downloaded sha1 v0.10.6
      Downloaded matchers v0.2.0
      Downloaded mime v0.3.17
      Downloaded base64ct v1.8.3
      Downloaded zeroize v1.8.2
      Downloaded tokio-rustls v0.26.4
      Downloaded anyhow v1.0.102
      Downloaded time-macros v0.2.27
      Downloaded equivalent v1.0.2
      Downloaded unicase v2.9.0
      Downloaded whoami v1.6.1
      Downloaded tracing-attributes v0.1.31
      Downloaded zune-core v0.5.1
      Downloaded num-conv v0.2.1
      Downloaded sqlx-core v0.8.6
      Downloaded futures-executor v0.3.32
      Downloaded generic-array v0.14.7
      Downloaded color_quant v1.1.0
      Downloaded bitflags v2.11.1
      Downloaded cpufeatures v0.2.17
      Downloaded crypto-common v0.1.7
      Downloaded funty v2.0.0
      Downloaded weezl v0.1.12
      Downloaded yoke v0.8.2
      Downloaded ahash v0.7.8
      Downloaded axum-extra v0.9.6
      Downloaded tokio-stream v0.1.18
      Downloaded hyper-rustls v0.27.9
      Downloaded ipnet v2.12.0
      Downloaded itoa v1.0.18
      Downloaded rand_chacha v0.3.1
      Downloaded bytemuck v1.25.0
      Downloaded unicode-properties v0.1.4
      Downloaded hashlink v0.10.0
      Downloaded scopeguard v1.2.0
      Downloaded cfg_aliases v0.2.1
      Downloaded toml_parser v1.1.2+spec-1.1.0
      Downloaded rend v0.4.2
      Downloaded signature v2.2.0
      Downloaded parking v2.2.1
      Downloaded sqlx-mysql v0.8.6
      Downloaded matchit v0.7.3
      Downloaded hex v0.4.3
      Downloaded futures-macro v0.3.32
      Downloaded tinyvec v1.11.0
      Downloaded borsh-derive v1.6.1
      Downloaded futures-core v0.3.32
      Downloaded serde_urlencoded v0.7.1
      Downloaded unicode-bidi v0.3.18
      Downloaded litemap v0.8.2
      Downloaded rand_core v0.6.4
      Downloaded mime_guess v2.0.5
      Downloaded num-integer v0.1.46
      Downloaded pkg-config v0.3.33
      Downloaded fdeflate v0.3.7
      Downloaded rustversion v1.0.22
      Downloaded seahash v4.1.0
      Downloaded nu-ansi-term v0.50.3
      Downloaded gif v0.14.2
      Downloaded getrandom v0.3.4
      Downloaded getrandom v0.2.17
      Downloaded signal-hook-registry v1.4.8
      Downloaded futures-task v0.3.32
      Downloaded quote v1.0.45
      Downloaded md-5 v0.10.6
      Downloaded http-body v1.0.1
      Downloaded multer v3.1.0
      Downloaded tracing-core v0.1.36
      Downloaded uuid v1.23.1
      Downloaded base64 v0.22.1
      Downloaded allocator-api2 v0.2.21
      Downloaded unicode-ident v1.0.24
      Downloaded lru-slab v0.1.2
      Downloaded futures-io v0.3.32
      Downloaded lock_api v0.4.14
      Downloaded once_cell v1.21.4
      Downloaded httpdate v1.0.3
      Downloaded idna_adapter v1.2.2
      Downloaded url v2.5.8
      Downloaded sqlx-sqlite v0.8.6
      Downloaded crc v3.4.0
      Downloaded parking_lot v0.12.5
      Downloaded crc-catalog v2.5.0
      Downloaded httparse v1.10.1
      Downloaded find-msvc-tools v0.1.9
      Downloaded toml_edit v0.25.11+spec-1.1.0
      Downloaded simd-adler32 v0.3.9
      Downloaded zerotrie v0.2.4
      Downloaded shlex v1.3.0
      Downloaded fastrand v2.4.1
      Downloaded typenum v1.20.0
      Downloaded icu_properties v2.2.0
      Downloaded form_urlencoded v1.2.2
      Downloaded serde_path_to_error v0.1.20
      Downloaded icu_normalizer v2.2.0
      Downloaded displaydoc v0.2.5
      Downloaded borsh v1.6.1
      Downloaded getrandom v0.4.2
      Downloaded sha2 v0.10.9
      Downloaded tower v0.5.3
      Downloaded simdutf8 v0.1.5
      Downloaded zune-jpeg v0.5.15
      Downloaded tower-http v0.5.2
      Downloaded concurrent-queue v2.5.0
      Downloaded icu_normalizer_data v2.2.0
      Downloaded icu_collections v2.2.0
      Downloaded dotenvy v0.15.7
      Downloaded slab v0.4.12
      Downloaded digest v0.10.7
      Downloaded sqlx-postgres v0.8.6
      Downloaded zerovec v0.11.6
      Downloaded futures-channel v0.3.32
      Downloaded unicode-normalization v0.1.25
      Downloaded tokio-util v0.7.18
      Downloaded http v1.4.0
      Downloaded memchr v2.8.0
      Downloaded tower-http v0.6.9
      Downloaded http-body-util v0.1.3
      Downloaded hyper-util v0.1.20
      Downloaded axum v0.7.9
      Downloaded crc32fast v1.5.0
      Downloaded rustls-pki-types v1.14.1
      Downloaded aho-corasick v1.1.4
      Downloaded winnow v1.0.2
      Downloaded num-bigint-dig v0.8.6
      Downloaded sqlx v0.8.6
      Downloaded hashbrown v0.17.0
      Downloaded rand_chacha v0.9.0
      Downloaded hashbrown v0.15.5
      Downloaded idna v1.1.0
      Downloaded spin v0.9.8
      Downloaded crossbeam-utils v0.8.21
      Downloaded deranged v0.5.8
      Downloaded foldhash v0.1.5
      Downloaded tracing-subscriber v0.3.23
      Downloaded smallvec v1.15.1
      Downloaded icu_properties_data v2.2.0
      Downloaded const-oid v0.9.6
      Downloaded cookie v0.18.1
      Downloaded either v1.15.0
      Downloaded moxcms v0.8.1
      Downloaded futures-util v0.3.32
      Downloaded libm v0.2.16
      Downloaded webpki-roots v1.0.7
      Downloaded syn v1.0.109
      Downloaded bitvec v1.0.1
      Downloaded time v0.3.47
      Downloaded vcpkg v0.2.15
      Downloaded quinn-udp v0.5.14
      Downloaded syn v2.0.117
      Downloaded zerocopy v0.8.48
      Downloaded proc-macro2 v1.0.106
      Downloaded image v0.25.10
      Downloaded iana-time-zone v0.1.65
      Downloaded event-listener v5.4.1
      Downloaded hmac v0.12.1
      Downloaded tracing v0.1.44
      Downloaded sharded-slab v0.1.7
      Downloaded socket2 v0.6.3
      Downloaded serde_core v1.0.228
      Downloaded serde_derive v1.0.228
      Downloaded tokio v1.52.2
      Downloaded icu_provider v2.2.0
      Downloaded log v0.4.29
      Downloaded bytes v1.11.1
      Downloaded ryu v1.0.23
      Downloaded flume v0.11.1
      Downloaded quinn v0.11.9
      Downloaded miniz_oxide v0.8.9
      Downloaded der v0.7.10
      Downloaded rustls-webpki v0.103.13
      Downloaded flate2 v1.1.9
      Downloaded rand v0.8.6
      Downloaded serde v1.0.228
      Downloaded rsa v0.9.10
      Downloaded icu_locale_core v2.2.0
      Downloaded futures-intrusive v0.5.0
      Downloaded image-webp v0.2.4
      Downloaded rand v0.9.4
      Downloaded cc v1.2.61
      Downloaded mio v1.2.0
      Downloaded hashbrown v0.12.3
      Downloaded rkyv v0.7.46
      Downloaded png v0.18.1
      Downloaded libsqlite3-sys v0.30.1
      Downloaded indexmap v2.14.0
      Downloaded serde_json v1.0.149
      Downloaded rust_decimal v1.41.0
      Downloaded hyper v1.9.0
      Downloaded hkdf v0.12.4
      Downloaded reqwest v0.12.28
      Downloaded quinn-proto v0.11.14
      Downloaded chrono v0.4.44
      Downloaded regex-syntax v0.8.10
      Downloaded rustls v0.23.40
      Downloaded regex-automata v0.4.14
      Downloaded pxfm v0.1.29
      Downloaded libc v0.2.186
      Downloaded encoding_rs v0.8.35
      Downloaded ring v0.17.14
       Compiling proc-macro2 v1.0.106
       Compiling unicode-ident v1.0.24
       Compiling quote v1.0.45
       Compiling libc v0.2.186
       Compiling version_check v0.9.5
       Compiling cfg-if v1.0.4
       Compiling serde_core v1.0.228
       Compiling autocfg v1.5.0
       Compiling libm v0.2.16
       Compiling shlex v1.3.0
       Compiling generic-array v0.14.7
       Compiling find-msvc-tools v0.1.9
       Compiling pin-project-lite v0.2.17
       Compiling futures-core v0.3.32
       Compiling icu_normalizer_data v2.2.0
       Compiling cc v1.2.61
       Compiling icu_properties_data v2.2.0
       Compiling serde v1.0.228
       Compiling bytes v1.11.1
       Compiling smallvec v1.15.1
       Compiling itoa v1.0.18
       Compiling futures-sink v0.3.32
       Compiling stable_deref_trait v1.2.1
       Compiling once_cell v1.21.4
       Compiling syn v2.0.117
       Compiling crossbeam-utils v0.8.21
       Compiling zerocopy v0.8.48
       Compiling errno v0.3.14
       Compiling const-oid v0.9.6
       Compiling zeroize v1.8.2
       Compiling signal-hook-registry v1.4.8
       Compiling socket2 v0.6.3
       Compiling mio v1.2.0
       Compiling memchr v2.8.0
       Compiling parking_lot_core v0.9.12
       Compiling ring v0.17.14
       Compiling scopeguard v1.2.0
       Compiling typenum v1.20.0
       Compiling lock_api v0.4.14
       Compiling getrandom v0.2.17
       Compiling tracing-core v0.1.36
       Compiling subtle v2.6.1
       Compiling futures-io v0.3.32
       Compiling slab v0.4.12
       Compiling futures-task v0.3.32
       Compiling rustls v0.23.40
       Compiling log v0.4.29
       Compiling futures-util v0.3.32
       Compiling num-traits v0.2.19
       Compiling percent-encoding v2.3.2
       Compiling block-buffer v0.10.4
       Compiling crypto-common v0.1.7
       Compiling digest v0.10.7
       Compiling writeable v0.6.3
       Compiling litemap v0.8.2
       Compiling thiserror v2.0.18
       Compiling spin v0.9.8
       Compiling http v1.4.0
       Compiling zmij v1.0.21
       Compiling utf8_iter v1.0.4
       Compiling synstructure v0.13.2
       Compiling rustls-pki-types v1.14.1
       Compiling num-bigint-dig v0.8.6
       Compiling http-body v1.0.1
       Compiling form_urlencoded v1.2.2
       Compiling base64ct v1.8.3
       Compiling untrusted v0.9.0
       Compiling pkg-config v0.3.33
       Compiling vcpkg v0.2.15
       Compiling pem-rfc7468 v0.7.0
       Compiling zerofrom-derive v0.1.7
       Compiling yoke-derive v0.8.2
       Compiling zerovec-derive v0.11.3
       Compiling displaydoc v0.2.5
       Compiling tracing-attributes v0.1.31
       Compiling serde_derive v1.0.228
       Compiling tokio-macros v2.7.0
       Compiling thiserror-impl v2.0.18
       Compiling zerofrom v0.1.7
       Compiling tokio v1.52.2
       Compiling yoke v0.8.2
       Compiling zerovec v0.11.6
       Compiling tracing v0.1.44
       Compiling zerotrie v0.2.4
       Compiling tinystr v0.8.3
       Compiling icu_locale_core v2.2.0
       Compiling potential_utf v0.1.5
       Compiling icu_collections v2.2.0
       Compiling libsqlite3-sys v0.30.1
       Compiling rand_core v0.6.4
       Compiling icu_provider v2.2.0
       Compiling icu_normalizer v2.2.0
       Compiling icu_properties v2.2.0
       Compiling futures-channel v0.3.32
       Compiling base64 v0.22.1
       Compiling equivalent v1.0.2
       Compiling tower-service v0.3.3
       Compiling httparse v1.10.1
       Compiling idna_adapter v1.2.2
       Compiling der v0.7.10
       Compiling foldhash v0.1.5
       Compiling allocator-api2 v0.2.21
       Compiling cpufeatures v0.2.17
       Compiling ppv-lite86 v0.2.21
       Compiling rust_decimal v1.41.0
       Compiling rustls-webpki v0.103.13
       Compiling rand_chacha v0.3.1
       Compiling spki v0.7.3
       Compiling hashbrown v0.15.5
       Compiling parking_lot v0.12.5
       Compiling idna v1.1.0
       Compiling concurrent-queue v2.5.0
       Compiling webpki-roots v1.0.7
       Compiling num-integer v0.1.46
       Compiling bitflags v2.11.1
       Compiling httpdate v1.0.3
       Compiling try-lock v0.2.5
       Compiling arrayvec v0.7.6
       Compiling tower-layer v0.3.3
       Compiling tinyvec_macros v0.1.1
       Compiling iana-time-zone v0.1.65
       Compiling hashbrown v0.17.0
       Compiling crc-catalog v2.5.0
       Compiling parking v2.2.1
       Compiling event-listener v5.4.1
       Compiling indexmap v2.14.0
       Compiling crc v3.4.0
       Compiling chrono v0.4.44
       Compiling tinyvec v1.11.0
       Compiling tokio-stream v0.1.18
       Compiling want v0.3.1
       Compiling num-iter v0.1.45
       Compiling webpki-roots v0.26.11
       Compiling url v2.5.8
       Compiling futures-intrusive v0.5.0
       Compiling hashlink v0.10.0
       Compiling rand v0.8.6
       Compiling pkcs8 v0.10.2
       Compiling sha2 v0.10.9
       Compiling either v1.15.0
       Compiling lazy_static v1.5.0
       Compiling crossbeam-queue v0.3.12
       Compiling sync_wrapper v1.0.2
       Compiling ryu v1.0.23
       Compiling mime v0.3.17
       Compiling atomic-waker v1.1.2
       Compiling hyper v1.9.0
       Compiling serde_urlencoded v0.7.1
       Compiling pkcs1 v0.7.5
       Compiling unicode-normalization v0.1.25
       Compiling sqlx-core v0.8.6
       Compiling signature v2.2.0
       Compiling http-body-util v0.1.3
       Compiling atoi v2.0.0
       Compiling hmac v0.12.1
       Compiling unicode-bidi v0.3.18
       Compiling rustversion v1.0.22
       Compiling serde_json v1.0.149
       Compiling unicode-properties v0.1.4
       Compiling simd-adler32 v0.3.9
       Compiling ipnet v2.12.0
       Compiling crc32fast v1.5.0
       Compiling hyper-util v0.1.20
       Compiling stringprep v0.1.5
       Compiling hkdf v0.12.4
       Compiling rsa v0.9.10
       Compiling tower v0.5.3
       Compiling futures-executor v0.3.32
       Compiling sha1 v0.10.6
       Compiling flume v0.11.1
       Compiling md-5 v0.10.6
       Compiling multer v3.1.0
       Compiling num-conv v0.2.1
       Compiling time-core v0.1.8
       Compiling adler2 v2.0.1
       Compiling unicase v2.9.0
       Compiling whoami v1.6.1
       Compiling powerfmt v0.2.0
       Compiling dotenvy v0.15.7
       Compiling hex v0.4.3
       Compiling byteorder v1.5.0
       Compiling deranged v0.5.8
       Compiling mime_guess v2.0.5
       Compiling time-macros v0.2.27
       Compiling sqlx-mysql v0.8.6
       Compiling miniz_oxide v0.8.9
       Compiling async-trait v0.1.89
       Compiling sqlx-sqlite v0.8.6
       Compiling cookie v0.18.1
       Compiling encoding_rs v0.8.35
       Compiling heck v0.5.0
       Compiling regex-syntax v0.8.10
       Compiling getrandom v0.4.2
       Compiling time v0.3.47
       Compiling sqlx-macros-core v0.8.6
       Compiling regex-automata v0.4.14
       Compiling axum-core v0.4.5
       Compiling flate2 v1.1.9
       Compiling fdeflate v0.3.7
       Compiling tokio-rustls v0.26.4
       Compiling tokio-util v0.7.18
       Compiling serde_path_to_error v0.1.20
       Compiling color_quant v1.1.0
       Compiling weezl v0.1.12
       Compiling byteorder-lite v0.1.0
       Compiling zune-core v0.5.1
       Compiling quick-error v2.0.1
       Compiling anyhow v1.0.102
       Compiling pxfm v0.1.29
       Compiling matchit v0.7.3
       Compiling image-webp v0.2.4
       Compiling axum v0.7.9
       Compiling zune-jpeg v0.5.15
       Compiling gif v0.14.2
       Compiling png v0.18.1
       Compiling moxcms v0.8.1
       Compiling hyper-rustls v0.27.9
       Compiling matchers v0.2.0
       Compiling sqlx-macros v0.8.6
       Compiling tower-http v0.6.9
       Compiling sharded-slab v0.1.7
       Compiling tracing-log v0.2.0
       Compiling thread_local v1.1.9
       Compiling http-range-header v0.4.2
       Compiling fastrand v2.4.1
       Compiling nu-ansi-term v0.50.3
       Compiling bytemuck v1.25.0
       Compiling tracing-subscriber v0.3.23
       Compiling axum-extra v0.9.6
       Compiling image v0.25.10
       Compiling tower-http v0.5.2
       Compiling reqwest v0.12.28
       Compiling uuid v1.23.1
       Compiling sqlx v0.8.6
       Compiling partkeepr-api v0.1.13 (/home/alan/partkeepr-ng/backend)
        Finished `release` profile [optimized] target(s) in 2m 14s
    ✓ • 2m 14s

## Activity

    $ cp target/release/partkeepr-api local-install/bin/partkeepr-api; ldd local-install/bin/partkeepr-api
        linux-vdso.so.1 (0x00007ffdb54df000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f7338ad3000)
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007f73389f4000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f7337a1f000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f7338b12000)
    ✓ • 0ms

## Assistant

The release build succeeded, and the runtime container is being prepared. Both new services are configured to restart automatically, and I’ve saved operating and backup instructions under `local-install/`. The original PartKeepr containers and source backups remain unchanged.

## Activity

    $ docker compose -f local-install/compose.yaml up -d --build app
    #0 building with "default" instance using docker driver
    
    #1 [app internal] load build definition from Dockerfile
    #1 transferring dockerfile: 273B done
    #1 DONE 0.0s
    
    #2 [app internal] load metadata for docker.io/library/debian:bookworm-slim
    #2 DONE 0.5s
    
    #3 [app internal] load .dockerignore
    #3 transferring context: 79B done
    #3 DONE 0.0s
    
    #4 [app 1/3] FROM docker.io/library/debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587
    #4 resolve docker.io/library/debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587 0.0s done
    #4 ...
    
    #5 [app internal] load build context
    #5 transferring context: 19.71MB 0.1s done
    #5 DONE 0.1s
    
    #4 [app 1/3] FROM docker.io/library/debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 2.10MB / 28.25MB 0.3s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 5.24MB / 28.25MB 0.6s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 7.34MB / 28.25MB 0.8s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 10.49MB / 28.25MB 1.1s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 12.58MB / 28.25MB 1.2s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 15.73MB / 28.25MB 1.5s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 17.83MB / 28.25MB 1.7s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 20.97MB / 28.25MB 2.0s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 23.07MB / 28.25MB 2.1s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 25.17MB / 28.25MB 2.3s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 28.25MB / 28.25MB 2.6s
    #4 sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 28.25MB / 28.25MB 2.6s done
    #4 extracting sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39
    #4 extracting sha256:7fd1ebe418b5079d4aa36f416af9b143d127bc75b0155d71cb5fd559b5e04d39 0.8s done
    #4 DONE 3.4s
    
    #6 [app 2/3] RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
    #6 0.433 Get:1 http://deb.debian.org/debian bookworm InRelease [151 kB]
    #6 0.483 Get:2 http://deb.debian.org/debian bookworm-updates InRelease [55.4 kB]
    #6 0.503 Get:3 http://deb.debian.org/debian-security bookworm-security InRelease [34.8 kB]
    #6 0.548 Get:4 http://deb.debian.org/debian bookworm/main amd64 Packages [8790 kB]
    #6 1.351 Get:5 http://deb.debian.org/debian bookworm-updates/main amd64 Packages [6924 B]
    #6 1.351 Get:6 http://deb.debian.org/debian-security bookworm-security/main amd64 Packages [353 kB]
    #6 2.241 Fetched 9392 kB in 2s (4968 kB/s)
    #6 2.241 Reading package lists...
    #6 2.719 Reading package lists...
    #6 3.183 Building dependency tree...
    #6 3.288 Reading state information...
    #6 3.425 The following additional packages will be installed:
    #6 3.426   libssl3 openssl
    #6 3.445 The following NEW packages will be installed:
    #6 3.446   ca-certificates libssl3 openssl
    #6 3.501 0 upgraded, 3 newly installed, 0 to remove and 1 not upgraded.
    #6 3.501 Need to get 3643 kB of archives.
    #6 3.501 After this operation, 8955 kB of additional disk space will be used.
    #6 3.501 Get:1 http://deb.debian.org/debian-security bookworm-security/main amd64 libssl3 amd64 3.0.22-1~deb12u1 [2039 kB]
    #6 3.718 Get:2 http://deb.debian.org/debian-security bookworm-security/main amd64 openssl amd64 3.0.22-1~deb12u1 [1442 kB]
    #6 3.841 Get:3 http://deb.debian.org/debian-security bookworm-security/main amd64 ca-certificates all 20250419~deb12u1 [162 kB]
    #6 3.945 debconf: delaying package configuration, since apt-utils is not installed
    #6 3.965 Fetched 3643 kB in 0s (9077 kB/s)
    #6 3.981 Selecting previously unselected package libssl3:amd64.
    #6 3.981 (Reading database ...
    #6 3.986 Preparing to unpack .../libssl3_3.0.22-1~deb12u1_amd64.deb ...
    #6 3.990 Unpacking libssl3:amd64 (3.0.22-1~deb12u1) ...
    #6 4.133 Selecting previously unselected package openssl.
    #6 4.135 Preparing to unpack .../openssl_3.0.22-1~deb12u1_amd64.deb ...
    #6 4.136 Unpacking openssl (3.0.22-1~deb12u1) ...
    #6 4.242 Selecting previously unselected package ca-certificates.
    #6 4.244 Preparing to unpack .../ca-certificates_20250419~deb12u1_all.deb ...
    #6 4.246 Unpacking ca-certificates (20250419~deb12u1) ...
    #6 4.299 Setting up libssl3:amd64 (3.0.22-1~deb12u1) ...
    #6 4.305 Setting up openssl (3.0.22-1~deb12u1) ...
    #6 4.314 Setting up ca-certificates (20250419~deb12u1) ...
    #6 4.381 debconf: unable to initialize frontend: Dialog
    #6 4.381 debconf: (TERM is not set, so the dialog frontend is not usable.)
    #6 4.381 debconf: falling back to frontend: Readline
    #6 4.382 debconf: unable to initialize frontend: Readline
    #6 4.382 debconf: (Can't locate Term/ReadLine.pm in @INC (you may need to install the Term::ReadLine module) (@INC contains: /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.36.0 /usr/local/share/perl/5.36.0 /usr/lib/x86_64-linux-gnu/perl5/5.36 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl-base /usr/lib/x86_64-linux-gnu/perl/5.36 /usr/share/perl/5.36 /usr/local/lib/site_perl) at /usr/share/perl5/Debconf/FrontEnd/Readline.pm line 7.)
    #6 4.382 debconf: falling back to frontend: Teletype
    #6 4.782 Updating certificates in /etc/ssl/certs...
    #6 5.349 150 added, 0 removed; done.
    #6 5.374 Processing triggers for libc-bin (2.36-9+deb12u14) ...
    #6 5.386 Processing triggers for ca-certificates (20250419~deb12u1) ...
    #6 5.391 Updating certificates in /etc/ssl/certs...
    #6 5.834 0 added, 0 removed; done.
    #6 5.834 Running hooks in /etc/ca-certificates/update.d...
    #6 5.835 done.
    #6 DONE 6.1s
    
    #7 [app 3/3] COPY bin/partkeepr-api /usr/local/bin/partkeepr-api
    #7 DONE 0.1s
    
    #8 [app] exporting to image
    #8 exporting layers
    #8 exporting layers 0.8s done
    #8 exporting manifest sha256:0048d9d5cd8b0d2a3f0737500a1dc0fc7234e1cce2dbe2642a62b95a02853ed1 done
    #8 exporting config sha256:64581b9f35927036b611716d6364caacf0fd77b6192eb808027a9a0bc9b6f19a done
    #8 exporting attestation manifest sha256:c4c1399bacfbf2b99872efc2b139a88e6e847a30a777d4afc56d56d38abfa50f done
    #8 exporting manifest list sha256:767c3f84d108cd55ad7383744a078b749182b17fb0c390972a847f1453fcfe67 done
    #8 naming to docker.io/library/partkeepr-ng-app:latest done
    #8 unpacking to docker.io/library/partkeepr-ng-app:latest
    #8 unpacking to docker.io/library/partkeepr-ng-app:latest 0.2s done
    #8 DONE 1.0s
     Container partkeepr-ng-db-1  Running
     Container partkeepr-ng-app-1  Creating
     Container partkeepr-ng-app-1  Created
     Container partkeepr-ng-db-1  Waiting
     Container partkeepr-ng-db-1  Healthy
     Container partkeepr-ng-app-1  Starting
     Container partkeepr-ng-app-1  Started
    ✓ • 12.21s

## Activity

    $ sed -n '155,165p' backend/src/main.rs; rg -n 'async fn download|extension|size' backend/src/handlers/attachments.rs | tail -25
            .await?;
        Ok(())
    }
    
    async fn health() -> &'static str {
        "ok"
    }
    
    async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
    431:        extension: saved_file.extension,
    529:            "SELECT id, filename, originalname, mimetype, size, description, \
    530:                    extension, COALESCE(isImage, 0) AS is_image \
    537:            "SELECT id, filename, originalname, mimetype, size, description, \
    538:                    extension, {always_img} AS is_image \
    640:                    ({fk}, type, filename, originalname, mimetype, size, \
    641:                     description, extension, isImage, created) \
    650:            .bind(saved_file.size)
    651:            .bind(&saved_file.extension)
    661:                    ({fk}, type, filename, originalname, mimetype, size, \
    662:                     description, extension, created) \
    671:            .bind(saved_file.size)
    672:            .bind(&saved_file.extension)
    684:            size: saved_file.size,
    686:            extension: saved_file.extension,
    698:pub async fn download(
    711:        row.extension.as_deref(),
    719:pub async fn download_thumb(
    738:    // resize cost; subsequent requests serve the cached file directly.
    741:            &dir, &row.filename, row.extension.as_deref(),
    835:    // handles legacy rows where the DB extension column doesn't match
    839:        &dir, &row.filename, row.extension.as_deref(),
    865:    extension: Option<String>,
    878:            "SELECT filename, originalname, mimetype, extension, \
    886:            "SELECT filename, originalname, mimetype, extension, \
    ✓ • 0ms

## Activity

    $ python3 local-install/verify.py; docker compose -f local-install/compose.yaml logs --tail=25 app; docker compose -f local-install/compose.yaml ps
    {'path': '/health', 'status': 200, 'bytes': 2}
    {'path': '/', 'status': 200, 'bytes': 1090}
    {'path': '/api/me (unauthenticated)', 'status': 401}
    {'path': '/api/me', 'status': 200, 'bytes': 48}
    {'path': '/api/version', 'status': 200, 'bytes': 20}
    {'path': '/api/parts', 'status': 200, 'bytes': 29574}
    {'path': '/api/part_categories/tree', 'status': 200, 'bytes': 27590}
    {'path': '/api/storage_tree', 'status': 200, 'bytes': 47090}
    {'path': '/api/footprint_tree', 'status': 200, 'bytes': 14752}
    {'path': '/api/projects', 'status': 200, 'bytes': 2}
    {'path': '/api/parts/1', 'status': 200, 'bytes': 1336}
    {'path': '/files/PartAttachment/1', 'status': 200, 'bytes': 324405}
    {'path': '/files/FootprintAttachment/1', 'status': 200, 'bytes': 138512}
    {'path': '/files/FootprintImage/1', 'status': 200, 'bytes': 23365}
    {'path': '/files/FootprintImage/1/thumb', 'status': 200, 'bytes': 10066}
    {'path': '/files/ManufacturerICLogo/1', 'status': 200, 'bytes': 2195}
    {'path': '/files/ManufacturerICLogo/1/thumb', 'status': 200, 'bytes': 3718}
    {'path': '/files/StorageLocationImage/1', 'status': 200, 'bytes': 3927785}
    {'path': '/files/StorageLocationImage/1/thumb', 'status': 200, 'bytes': 10860}
    app-1  | 2026-10-16T10:36:24.214679Z  INFO partkeepr_api: connected to database
    app-1  | 2026-10-16T10:36:24.217589Z  INFO partkeepr_api::startup: created convention bin name=(NOWHERE)
    app-1  | 2026-10-16T10:36:24.217635Z  INFO partkeepr_api::handlers::storage: file storage root root=/storage
    app-1  | 2026-10-16T10:36:24.217660Z  WARN partkeepr_api::handlers::clamd: virus scanning disabled — set PARTKEEPR_CLAMD=tcp://host:port to enable
    app-1  | 2026-10-16T10:36:24.217680Z  INFO partkeepr_api::handlers::printing: ptouch-print disabled (set PARTKEEPR_PTOUCH_BIN to enable)
    app-1  | 2026-10-16T10:36:24.217686Z  INFO partkeepr_api::handlers::mouser: mouser search api disabled (set PARTKEEPR_MOUSER_SEARCH_KEY to enable)
    app-1  | 2026-10-16T10:36:24.217693Z  INFO partkeepr_api::handlers::digikey: digikey api disabled (set PARTKEEPR_DIGIKEY_CLIENT_ID and PARTKEEPR_DIGIKEY_CLIENT_SECRET to enable)
    app-1  | 2026-10-16T10:36:24.217698Z  INFO partkeepr_api::handlers::trustedparts: trustedparts api disabled (set PARTKEEPR_TRUSTEDPARTS_API_KEY to enable)
    app-1  | 2026-10-16T10:36:24.217702Z  INFO partkeepr_api::handlers::jlcparts: jlcparts cross-reference disabled (set PARTKEEPR_JLCPARTS_DB_PATH to the mirrored cache.sqlite3 to enable)
    app-1  | 2026-10-16T10:36:24.217723Z  INFO partkeepr_api: feature status — clamd=off ptouch=off mouser_search=off mouser_orders=off digikey=off trustedparts=off jlcparts=off
    app-1  | 2026-10-16T10:36:24.217748Z  INFO partkeepr_api: serving static frontend frontend_dir="/frontend"
    app-1  | 2026-10-16T10:36:24.219260Z  INFO partkeepr_api: listening bind_addr=0.0.0.0:8080
    NAME                 IMAGE              COMMAND                  SERVICE   CREATED          STATUS                   PORTS
    partkeepr-ng-app-1   partkeepr-ng-app   "/usr/local/bin/part…"   app       13 seconds ago   Up 12 seconds            127.0.0.1:8082->8080/tcp
    partkeepr-ng-db-1    mariadb:10.11      "docker-entrypoint.s…"   db        3 minutes ago    Up 3 minutes (healthy)   3306/tcp
    ✓ • 1.30s

## Assistant

Installed and running at **[http://localhost:8082/](http://localhost:8082/)** on this machine. Sign in with your existing PartKeepr credentials.

- Imported **669 parts** and **605 stock-history entries**.
- Applied all six migrations.
- Imported attachments, footprint images, manufacturer logos, and storage images.
- Confirmed **every database file reference has a matching file**—no additional extraction is needed.
- Verified API reads, sample downloads, and thumbnail generation.

The new installation uses separate Docker containers and persistent storage, with automatic restart enabled. Your original PartKeepr installation and backups remain unchanged.

Password login still needs your check; I validated authenticated requests using a temporary session without changing your password. Optional supplier integrations remain disabled.

Operating and backup instructions: [local-install/README.md](/home/alan/partkeepr-ng/local-install/README.md).

## User

Right now I am working on remotely from the machine. I'm using Wireguard VPN to get to the machine, but localhost:8082 is not found, not if I use 192.168.x.y:8082

## Assistant

The app is bound to the server’s localhost, so it can’t accept connections through WireGuard. I’ll check the server’s network addresses and update the port binding for remote access.

## Activity

    $ ip -brief address; cat local-install/compose.yaml
    Cannot open netlink socket: Operation not permitted
    name: partkeepr-ng
    services:
      db:
        image: mariadb:10.11
        restart: unless-stopped
        env_file: database.env
        volumes:
          - database:/var/lib/mysql
        healthcheck:
          test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
          interval: 5s
          timeout: 5s
          retries: 30
      app:
        build:
          context: .
          dockerfile: Dockerfile
        restart: unless-stopped
        user: "1000:1000"
        env_file: app.env
        ports:
          - "127.0.0.1:8082:8080"
        volumes:
          - ./storage:/storage
          - ../frontend:/frontend:ro
        depends_on:
          db:
            condition: service_healthy
    volumes:
      database:
    ✓ • 0ms

## Activity

    $ ip -brief address; ip route; ss -ltn '( sport = :8082 )'
    lo               UNKNOWN        127.0.0.1/8 ::1/128
    eno1             UP             192.168.5.109/22 fd35:d43f:3135:1:7bae:e0be:739:cad/64 fd35:d43f:3135:1:54e9:8727:df0c:5e22/64 fd35:d43f:3135:1:18c9:3b9b:74cf:7ed5/64 fd35:d43f:3135:1:86cd:c00a:30f0:eed9/64 fd35:d43f:3135:1:58f2:3133:c636:7d85/64 fd35:d43f:3135:1:963e:2d98:174:4810/64 fd35:d43f:3135:1:db2d:959f:90b3:97a1/64 fd35:d43f:3135:1:1e66:6dff:fe93:3ca0/64 fe80::1e66:6dff:fe93:3ca0/64
    wg0              UNKNOWN        10.0.0.1/24
    br-81684cbd1062  UP             172.18.0.1/16 fe80::6420:79ff:fe64:5fc0/64
    br-cffb0944697f  UP             172.19.0.1/16 fe80::44e0:23ff:fe08:b383/64
    br-e2261d85aa8f  UP             172.21.0.1/16 fe80::5cb2:8bff:fe64:e716/64
    br-4ea3168253d7  UP             172.20.0.1/16 fe80::90cf:d3ff:feee:a9c/64
    br-63ca325b81f7  UP             172.23.0.1/16 fe80::6035:64ff:fe98:2a87/64
    docker0          DOWN           172.17.0.1/16 fe80::bc0a:a6ff:fe57:7ab6/64
    br-74c2b4cb9101  DOWN           172.22.0.1/16
    br-2874ccba9663  UP             172.24.0.1/16 fe80::7002:8dff:feb2:f5f7/64
    br-fb2e9998e0fa  UP             172.25.0.1/16 fe80::7c24:faff:fe83:ef4e/64
    veth9534e7d@if2  UP             fe80::403f:2eff:feb7:94e6/64
    vetha6ec459@if2  UP             fe80::94dc:7cff:feb3:ecb3/64
    veth830d3f6@if2  UP             fe80::c94:9ff:fed5:e959/64
    veth2b98586@if2  UP             fe80::e43d:aeff:fe0d:1080/64
    veth46d187a@if2  UP             fe80::c431:6dff:feab:6927/64
    veth150108a@if2  UP             fe80::9097:6fff:fe06:87c1/64
    vethde26dba@if2  UP             fe80::bc82:e2ff:fe0c:3596/64
    veth77af548@if2  UP             fe80::64df:b8ff:fe06:7f3/64
    veth19567c8@if2  UP             fe80::a442:2bff:fe03:8c56/64
    veth3f56af5@if2  UP             fe80::18f3:ffff:fed0:326e/64
    vetha64f243@if2  UP             fe80::8468:e3ff:fe2d:da82/64
    veth228557e@if2  UP             fe80::cba:5aff:fe50:35dc/64
    br-aaa22a1da2c7  UP             172.26.0.1/16 fe80::1c31:c6ff:fe96:1e39/64
    veth085e378@if2  UP             fe80::9829:f4ff:fee9:1431/64
    br-1f9de2359ac9  UP             172.27.0.1/16 fe80::1c66:edff:fe40:3469/64
    veth8ba5da4@if2  UP             fe80::b0b3:b2ff:fefc:9768/64
    veth5bb8910@if2  UP             fe80::5848:9fff:fecb:a82c/64
    vethd70472a@if2  UP             fe80::a487:d9ff:feb1:2545/64
    br-4878dc4a00d6  UP             172.28.0.1/16 fe80::80a9:deff:fe10:5421/64
    veth3fe87a4@if2  UP             fe80::606f:5bff:fec9:c885/64
    veth6e1e0fb@if2  UP             fe80::ccf9:e4ff:fe75:a6a1/64
    default via 192.168.4.1 dev eno1 proto dhcp src 192.168.5.109 metric 100
    10.0.0.0/24 dev wg0 proto kernel scope link src 10.0.0.1
    172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown
    172.18.0.0/16 dev br-81684cbd1062 proto kernel scope link src 172.18.0.1
    172.19.0.0/16 dev br-cffb0944697f proto kernel scope link src 172.19.0.1
    172.20.0.0/16 dev br-4ea3168253d7 proto kernel scope link src 172.20.0.1
    172.21.0.0/16 dev br-e2261d85aa8f proto kernel scope link src 172.21.0.1
    172.22.0.0/16 dev br-74c2b4cb9101 proto kernel scope link src 172.22.0.1 linkdown
    172.23.0.0/16 dev br-63ca325b81f7 proto kernel scope link src 172.23.0.1
    172.24.0.0/16 dev br-2874ccba9663 proto kernel scope link src 172.24.0.1
    172.25.0.0/16 dev br-fb2e9998e0fa proto kernel scope link src 172.25.0.1
    172.26.0.0/16 dev br-aaa22a1da2c7 proto kernel scope link src 172.26.0.1
    172.27.0.0/16 dev br-1f9de2359ac9 proto kernel scope link src 172.27.0.1
    172.28.0.0/16 dev br-4878dc4a00d6 proto kernel scope link src 172.28.0.1
    192.168.4.0/22 dev eno1 proto kernel scope link src 192.168.5.109 metric 100
    State  Recv-Q Send-Q Local Address:Port Peer Address:PortProcess
    LISTEN 0      4096       127.0.0.1:8082      0.0.0.0:*
    ✓ • 0ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('local-install/compose.yaml')
        s=p.read_text().replace('      - "127.0.0.1:8082:8080"','      - "127.0.0.1:8082:8080"\n      - "10.0.0.1:8082:8080"\n      - "192.168.5.109:8082:8080"')
        p.write_text(s)
        p=Path('local-install/README.md');s=p.read_text().replace('URL: http://localhost:8082/ (bound to localhost).','URLs:\n- WireGuard: http://10.0.0.1:8082/\n- LAN: http://192.168.5.109:8082/\n- On the server: http://localhost:8082/\n\nCompose binds explicitly to these three IPv4 addresses. Update compose.yaml\nif the server LAN or VPN address changes.');p.write_text(s)
        PY
    ✓ • 9ms

## Activity

    $ docker compose -f local-install/compose.yaml up -d app; python3 - <<'PY'
        import urllib.request
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
        for host in ['127.0.0.1','10.0.0.1','192.168.5.109']:
         try:
          with opener.open('http://'+host+':8082/health',timeout=10) as r: print(host,r.status,r.read().decode())
         except Exception as e: print(host,str(e))
        PY
     Container partkeepr-ng-db-1  Running
     Container partkeepr-ng-app-1  Recreate
     Container partkeepr-ng-app-1  Recreated
     Container partkeepr-ng-db-1  Waiting
     Container partkeepr-ng-db-1  Healthy
     Container partkeepr-ng-app-1  Starting
     Container partkeepr-ng-app-1  Started
    127.0.0.1 200 ok
    10.0.0.1 200 ok
    192.168.5.109 200 ok
    ✓ • 11.25s

## Assistant

Fixed—the app now accepts connections on the server’s VPN and LAN addresses.

Use **[http://10.0.0.1:8082/](http://10.0.0.1:8082/)** over WireGuard, or [http://192.168.5.109:8082/](http://192.168.5.109:8082/) if your VPN routes the LAN subnet.

Both addresses passed server-side health checks. Sign in with your existing PartKeepr credentials.

`localhost` on your remote computer refers to that computer, rather than the server.

## User

I can load the app, but admin/admin credentials are not accepted

## Assistant

The imported database uses your original PartKeepr accounts; `admin/admin` only applies to a fresh database. I’ll check the imported admin account and login response to see whether this is a password mismatch or an account migration issue.

## Activity

    $ sed -n '1,210p' backend/src/bin/reset-password.rs; rg -n 'verify_fos_password|iterations' backend/src/handlers/auth.rs
    // SPDX-License-Identifier: GPL-3.0-or-later
    
    //! `cargo run --bin reset-password -- <username> <new-password>`
    //!
    //! Resets a single FOSUser row's password hash to a new value, using
    //! the same FOSUserBundle scheme the login handler verifies against
    //! (sha512 over `password{salt}` iterated 5000×, base64). Looks up
    //! by either `username` or `username_canonical` so casing doesn't
    //! matter.
    //!
    //! The DATABASE_URL is read from the environment (or `.env` via
    //! dotenvy) — same as the API server. Connect to the same database
    //! the running app uses; otherwise the reset won't take effect for
    //! the live login flow.
    //!
    //! Notes:
    //!   - Existing salt is preserved. FOSUserBundle doesn't require salt
    //!     rotation, and keeping it stable means failed-attempt timing
    //!     stays comparable to other accounts.
    //!   - Won't unlock or enable a user — only changes the password
    //!     hash. If `locked=1` or `enabled=0` the user still can't log
    //!     in; that's a separate admin task.
    //!   - Won't create a missing `PartKeeprUser` row. If a FOSUser has
    //!     no PartKeeprUser counterpart, the login handler refuses
    //!     login (per the existing code path); password reset alone
    //!     won't fix that.
    
    use base64::Engine as _;
    use sha2::{Digest, Sha512};
    use sqlx::mysql::MySqlPoolOptions;
    use std::env;
    use std::process::ExitCode;
    
    #[tokio::main]
    async fn main() -> ExitCode {
        dotenvy::dotenv().ok();
    
        let args: Vec<String> = env::args().collect();
        if args.len() != 3 {
            eprintln!("usage: reset-password <username> <new-password>");
            eprintln!();
            eprintln!("Example:");
            eprintln!("  cargo run --bin reset-password -- admin 'hunter2'");
            return ExitCode::from(2);
        }
        let username = &args[1];
        let new_password = &args[2];
    
        let database_url = match env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(_) => {
                eprintln!("error: DATABASE_URL must be set (see backend/.env)");
                return ExitCode::from(1);
            }
        };
    
        let pool = match MySqlPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: connecting to DB failed: {e}");
                return ExitCode::from(1);
            }
        };
    
        // Look up the user's salt + status. We accept either casing of
        // the username argument.
        let row: Option<(i64, String, bool, bool)> = match sqlx::query_as(
            "SELECT id, salt, enabled, locked \
             FROM FOSUser \
             WHERE username_canonical = LOWER(?) OR username = ? \
             LIMIT 1",
        )
        .bind(username)
        .bind(username)
        .fetch_optional(&pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: lookup query failed: {e}");
                return ExitCode::from(1);
            }
        };
    
        let (user_id, salt, enabled, locked) = match row {
            Some(r) => r,
            None => {
                eprintln!("error: user '{username}' not found in FOSUser");
                return ExitCode::from(1);
            }
        };
    
        let hashed = hash_fos_password(new_password, &salt);
    
        let res = match sqlx::query("UPDATE FOSUser SET password = ? WHERE id = ?")
            .bind(&hashed)
            .bind(user_id)
            .execute(&pool)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: UPDATE failed: {e}");
                return ExitCode::from(1);
            }
        };
    
        if res.rows_affected() != 1 {
            eprintln!(
                "error: UPDATE affected {} rows (expected 1)",
                res.rows_affected()
            );
            return ExitCode::from(1);
        }
    
        println!("password reset for '{username}' (FOSUser.id={user_id})");
        if !enabled {
            println!("note: user is currently disabled (enabled=0) — they still can't log in.");
        }
        if locked {
            println!("note: user is currently locked (locked=1) — they still can't log in.");
        }
        ExitCode::SUCCESS
    }
    
    /// FOSUserBundle's MessageDigestPasswordEncoder: salted = "{pw}{salt}",
    /// digest = sha512(salted), then 4999 more iterations of
    /// sha512(prev_digest || salted), final result base64-encoded.
    ///
    /// Mirrors `verify_fos_password` in `handlers/auth.rs`. Keep them in
    /// sync; if either is changed, both must change.
    fn hash_fos_password(password: &str, salt: &str) -> String {
        let salted = format!("{password}{{{salt}}}");
        let salted_bytes = salted.as_bytes();
        let mut digest = Sha512::digest(salted_bytes).to_vec();
        for _ in 1..5000 {
            let mut h = Sha512::new();
            h.update(&digest);
            h.update(salted_bytes);
            digest = h.finalize().to_vec();
        }
        base64::engine::general_purpose::STANDARD.encode(&digest)
    }
    147:/// for i in 1..5000:           # 4999 more iterations, 5000 total
    153:pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
    237:    let pw_ok = verify_fos_password(&req.password, &row.salt, &row.password);
    ✓ • 0ms

## Activity

    $ docker compose -f local-install/compose.yaml exec -T db sh -c 'exec mariadb -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE" -e "SELECT f.id,f.username,f.enabled,f.locked,LENGTH(f.password) AS hash_length,LENGTH(f.salt) AS salt_length,p.id AS app_user_id,p.active,p.admin FROM FOSUser f LEFT JOIN PartKeeprUser p ON p.username=f.username;"'; python3 - <<'PY'
        import urllib.request,urllib.error,json
        req=urllib.request.Request('http://127.0.0.1:8082/api/login',data=json.dumps({'username':'admin','password':'admin'}).encode(),headers={'Content-Type':'application/json'})
        try:
         with urllib.request.urlopen(req) as r: print('Login status:',r.status)
        except urllib.error.HTTPError as e: print('Login status:',e.code,'Response:',e.read().decode())
        PY
    id    username    enabled    locked    hash_length    salt_length    app_user_id    active    admin
    1    alan    1    0    88    31    1    1    0
    Login status: 401 Response: {"error":"username or password not recognised"}
    ✓ • 117ms

## Assistant

Your imported database has one login account: **`alan`**, which is enabled and unlocked. There is no `admin` account.

Sign in as **`alan` using your original PartKeepr password**. The migration preserved it.

If you’ve forgotten that password, I can reset it in the new installation.

## User

Nope I am using alan + alan's original PartKeepr password and it is not working on partkeepr-ng. I DOES still work on the og partkeepr.

## Assistant

I’ll compare partkeepr-ng’s password verification with the running original installation. Since the password works there, this may be a compatibility bug.

## Activity

    $ sed -n '135,180p' backend/src/handlers/auth.rs; rg -n 'password|sha512|salt' backend/src/handlers/auth.rs | tail -30; rg --files | rg '(test|security)'
    
    // ---------------------------------------------------------------------------
    //  FOSUserBundle password verification
    // ---------------------------------------------------------------------------
    
    /// Verify a candidate password against the stored FOSUser hash.
    ///
    /// FOSUserBundle's default `MessageDigestPasswordEncoder`:
    ///
    /// ```text
    /// salted = password + "{" + salt + "}"
    /// digest = sha512(salted)
    /// for i in 1..5000:           # 4999 more iterations, 5000 total
    ///     digest = sha512(digest || salted)
    /// stored = base64(digest)
    /// ```
    ///
    /// Returns true iff `password` produces `stored_hash_b64`.
    pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
        let salted = format!("{password}{{{salt}}}");
        let salted_bytes = salted.as_bytes();
        let mut digest = Sha512::digest(salted_bytes).to_vec();
        for _ in 1..5000 {
            let mut h = Sha512::new();
            h.update(&digest);
            h.update(salted_bytes);
            digest = h.finalize().to_vec();
        }
        let computed = base64::engine::general_purpose::STANDARD.encode(&digest);
        bool::from(computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
    }
    
    // ---------------------------------------------------------------------------
    //  Login handler
    // ---------------------------------------------------------------------------
    
    #[derive(Debug, Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }
    
    #[derive(Debug, Serialize)]
    pub struct LoginResponse {
        pub user: CurrentUserView,
    }
    5://! Login verifies against the legacy `FOSUser` table's password hash
    6://! (FOSUserBundle's default: sha512 over `password{salt}` iterated 5000
    137://  FOSUserBundle password verification
    140:/// Verify a candidate password against the stored FOSUser hash.
    145:/// salted = password + "{" + salt + "}"
    146:/// digest = sha512(salted)
    148:///     digest = sha512(digest || salted)
    152:/// Returns true iff `password` produces `stored_hash_b64`.
    153:pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
    154:    let salted = format!("{password}{{{salt}}}");
    155:    let salted_bytes = salted.as_bytes();
    156:    let mut digest = Sha512::digest(salted_bytes).to_vec();
    160:        h.update(salted_bytes);
    174:    pub password: String,
    202:    if username.is_empty() || req.password.is_empty() {
    204:            "username and password are required",
    213:        password: String,
    214:        salt: String,
    219:        "SELECT id, username, password, salt, enabled, locked \
    231:        password: "X".repeat(88),
    232:        salt: "X".repeat(31),
    237:    let pw_ok = verify_fos_password(&req.password, &row.salt, &row.password);
    241:            "username or password not recognised",
    docs/manual-smoke-tests.md
    ✓ • 0ms

## Activity

    $ docker exec partkeepr-partkeepr-1 sh -c 'find /app -name "MessageDigestPasswordEncoder.php" -o -name "security.yml" -o -name "security.yaml" -o -name "User.php" | head -25'
    /app/src/PartKeepr/AuthBundle/Entity/User.php
    /app/app/config/security.yml
    /app/vendor/liip/functional-test-bundle/Tests/App/Entity/User.php
    /app/vendor/ddeboer/data-import/tests/Ddeboer/DataImport/Tests/Fixtures/Entity/User.php
    /app/vendor/symfony/symfony/src/Symfony/Bridge/Doctrine/Tests/Fixtures/User.php
    /app/vendor/symfony/symfony/src/Symfony/Component/Security/Core/User/User.php
    /app/vendor/symfony/symfony/src/Symfony/Component/Security/Core/Encoder/MessageDigestPasswordEncoder.php
    /app/vendor/friendsofsymfony/user-bundle/Propel/User.php
    /app/vendor/friendsofsymfony/user-bundle/Model/User.php
    /app/vendor/doctrine/data-fixtures/tests/Doctrine/Tests/Common/DataFixtures/TestEntity/User.php
    /app/vendor/dunglas/api-bundle/Tests/Behat/TestBundle/Entity/User.php
    /app/vendor/dunglas/api-bundle/features/fixtures/TestApp/config/security.yml
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Sluggable/Fixture/Handler/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Sluggable/Fixture/Issue1058/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Tree/Fixture/Closure/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Tree/Fixture/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/SoftDeleteable/Fixture/Entity/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/SoftDeleteable/Fixture/Document/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Blameable/Fixture/Document/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Mapping/Fixture/Yaml/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Mapping/Fixture/User.php
    /app/vendor/gedmo/doctrine-extensions/tests/Gedmo/Mapping/Fixture/Document/User.php
    ✓ • 6.91s

## Activity

    $ docker exec partkeepr-partkeepr-1 cat /app/app/config/security.yml /app/vendor/symfony/symfony/src/Symfony/Component/Security/Core/Encoder/MessageDigestPasswordEncoder.php; docker exec partkeepr-partkeepr-1 sh -c 'rg -n "function (getPassword|getSalt)|password|legacy" /app/src/PartKeepr/AuthBundle/Entity/User.php /app/src/PartKeepr/AuthBundle/Services 2>/dev/null || grep -n "password\|legacy\|getSalt" /app/src/PartKeepr/AuthBundle/Entity/User.php'
    security:
        encoders:
            Symfony\Component\Security\Core\User\User:
              algorithm: sha512
              iterations: 1
            FOS\UserBundle\Model\UserInterface:
              algorithm: sha512
              iterations: 1
            PartKeepr\AuthBundle\Entity\User:
              algorithm: md5
              iterations: 1
              encode_as_base64: false
    
        role_hierarchy:
            ROLE_ADMIN:       ROLE_USER
            ROLE_SUPER_ADMIN: ROLE_ADMIN
    
        providers:
            chain_provider:
                chain:
                    providers: [in_memory, fos_userbundle, fr3d_ldapbundle, legacy]
            legacy:
                id: partkeepr_legacy_user_provider
            in_memory:
                memory: ~
            fos_userbundle:
                id: fos_user.user_provider.username
            fr3d_ldapbundle:
                id: fr3d_ldap.security.user.provider
    
        firewalls:
            login:
                pattern: ^/api/users/getSalt
                security: false
            main:
                stateless: false # Stores authentification within a cookie. This is important to allow file uploads without having to send authentification data.
                pattern: ^/api/.*
                provider: chain_provider
                fr3d_ldap_httpbasic:
                    provider: chain_provider
                http_basic:
                    provider: chain_provider
                remote_user:
                    provider: chain_provider
                wsse:
                    realm: "Secured with WSSE" #identifies the set of resources to which the authentication information will apply (WWW-Authenticate)
                    profile: "UsernameToken" #WSSE profile (WWW-Authenticate)
                    encoder: #digest algorithm
                        algorithm: sha512
                        encodeHashAsBase64:    true
                        iterations: 1
    
        access_control:
            - { path: ^/login$, role: IS_AUTHENTICATED_ANONYMOUSLY }
            - { path: ^/register, role: IS_AUTHENTICATED_ANONYMOUSLY }
            - { path: ^/resetting, role: IS_AUTHENTICATED_ANONYMOUSLY }
            - { path: ^/admin/, role: ROLE_ADMIN }
    <?php
    
    /*
     * This file is part of the Symfony package.
     *
     * (c) Fabien Potencier <fabien@symfony.com>
     *
     * For the full copyright and license information, please view the LICENSE
     * file that was distributed with this source code.
     */
    
    namespace Symfony\Component\Security\Core\Encoder;
    
    use Symfony\Component\Security\Core\Exception\BadCredentialsException;
    
    /**
     * MessageDigestPasswordEncoder uses a message digest algorithm.
     *
     * @author Fabien Potencier <fabien@symfony.com>
     */
    class MessageDigestPasswordEncoder extends BasePasswordEncoder
    {
        private $algorithm;
        private $encodeHashAsBase64;
        private $iterations;
    
        /**
         * @param string $algorithm          The digest algorithm to use
         * @param bool   $encodeHashAsBase64 Whether to base64 encode the password hash
         * @param int    $iterations         The number of iterations to use to stretch the password hash
         */
        public function __construct($algorithm = 'sha512', $encodeHashAsBase64 = true, $iterations = 5000)
        {
            $this->algorithm = $algorithm;
            $this->encodeHashAsBase64 = $encodeHashAsBase64;
            $this->iterations = $iterations;
        }
    
        /**
         * {@inheritdoc}
         */
        public function encodePassword($raw, $salt)
        {
            if ($this->isPasswordTooLong($raw)) {
                throw new BadCredentialsException('Invalid password.');
            }
    
            if (!in_array($this->algorithm, hash_algos(), true)) {
                throw new \LogicException(sprintf('The algorithm "%s" is not supported.', $this->algorithm));
            }
    
            $salted = $this->mergePasswordAndSalt($raw, $salt);
            $digest = hash($this->algorithm, $salted, true);
    
            // "stretch" hash
            for ($i = 1; $i < $this->iterations; ++$i) {
                $digest = hash($this->algorithm, $digest.$salted, true);
            }
    
            return $this->encodeHashAsBase64 ? base64_encode($digest) : bin2hex($digest);
        }
    
        /**
         * {@inheritdoc}
         */
        public function isPasswordValid($encoded, $raw, $salt)
        {
            return !$this->isPasswordTooLong($raw) && $this->comparePasswords($encoded, $this->encodePassword($raw, $salt));
        }
    }
    33:    private $password;
    58:     * Marks a user as a legacy user (=old md5 auth).
    65:    private $legacy;
    278:        return $this->password;
    282:     * Sets the user's password.
    284:     * @param string $password
    286:    public function setPassword($password)
    288:        $this->password = $password;
    297:     * Sets the new password. Used for password changes.
    299:     * @param string $password The new password
    301:    public function setNewPassword($password)
    303:        $this->newPassword = $password;
    307:     * Compares the given un-hashed password with the
    308:     * object's hashed password.
    311:     * @param string $password The unhashed password
    313:     * @return bool true if the passwords match, false otherwise
    315:    public function comparePassword($password)
    317:        return $this->compareHashedPassword(md5($password));
    321:     * Compares the given hashed password with the object's
    322:     * hashed password.
    324:     * @param string $hashedPassword The md5-hashed password
    326:     * @return bool true if the passwords match, false otherwise
    330:        if ($hashedPassword == $this->password) {
    346:     * Returns if the user is a legacy user.
    352:        return $this->legacy;
    356:     * Marks a user as a legacy user.
    358:     * @param bool|true $legacy
    360:    public function setLegacy($legacy = true)
    362:        $this->legacy = $legacy;
    380:    public function getSalt()
    ✓ • 0ms

## Activity

    $ tail -50 backend/src/handlers/auth.rs; docker exec partkeepr-partkeepr-1 php -r 'require "/app/vendor/autoload.php"; foreach ([1,5000] as $iterations) { $e = new Symfony\Component\Security\Core\Encoder\MessageDigestPasswordEncoder("sha512", true, $iterations); foreach (["fixture-salt", ""] as $salt) { echo $iterations, " ", json_encode($salt), " ", $e->encodePassword("migration-test-password", $salt), "\n"; }}'
    }
    
    // ---------------------------------------------------------------------------
    //  Middleware
    // ---------------------------------------------------------------------------
    
    /// Auth middleware. Runs ahead of every handler. For public paths
    /// (`/health`, `/api/login`) it passes through untouched. For everything
    /// else: extracts the session cookie, verifies it, and either attaches
    /// the `CurrentUser` to the request extensions (so handlers can pull
    /// it via `Extension<CurrentUser>`) or returns 401.
    pub async fn auth_middleware(
        State(signer): State<SessionSigner>,
        jar: CookieJar,
        mut req: Request,
        next: Next,
    ) -> Response {
        let path = req.uri().path();
    
        // Whitelist exact paths. Subpaths under /health or /api/login don't
        // exist; we only want to whitelist the actual endpoints.
        if PUBLIC_PATHS.iter().any(|p| *p == path) {
            return next.run(req).await;
        }
    
        // Static frontend assets (everything outside /api/* and /files/*) are
        // served by tower-http's ServeDir fallback and don't require auth —
        // the login page itself has to be reachable for the user to log in.
        // /api/* routes always require auth (except /api/login above);
        // /files/* (slice 7 attachment downloads) also requires auth.
        if !path.starts_with("/api/") && !path.starts_with("/files/") {
            return next.run(req).await;
        }
    
        let cookie = jar.get(SESSION_COOKIE);
        match cookie.and_then(|c| signer.decode(c.value())) {
            Some(user) => {
                req.extensions_mut().insert(user);
                next.run(req).await
            }
            None => {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "not authenticated"})),
                )
                    .into_response()
            }
        }
    }
    
    1 "fixture-salt" ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ==
    1 "" 56Pet5rfF3IeVtwi7PY/i/RYiPiK3LqjvHoKZxDlneP760AzH47ELwwL7DA3dlezZ7nLYA8ET6scqvGGfms9sw==
    5000 "fixture-salt" yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw==
    5000 "" t6IMG9m1WloOJoNUlEHdxZncGEmOgYKjuZ2Cc132ouOL3jVIwlMBJ0h1Q48SzCSBR81PvCMFzI8Y0kwS/v0dDA==
    ✓ • 0ms

## Assistant

I found a compatibility bug: your original PartKeepr uses one SHA-512 hashing iteration, while partkeepr-ng assumes 5,000. That rejects the same password even with the correct imported hash.

I’ll add support for the original format while preserving support for passwords created by partkeepr-ng, test it against the original Symfony encoder, and redeploy. Your password won’t need to change.

## Activity

    file changes: Completed · 2 changes
    Update { move_path: None }: /home/alan/partkeepr-ng/backend/src/bin/reset-password.rs
    @@ -136,3 +136,7 @@
     fn hash_fos_password(password: &str, salt: &str) -> String {
    -    let salted = format!("{password}{{{salt}}}");
    +    let salted = if salt.is_empty() {
    +        password.to_owned()
    +    } else {
    +        format!("{password}{{{salt}}}")
    +    };
         let salted_bytes = salted.as_bytes();
    Update { move_path: None }: /home/alan/partkeepr-ng/backend/src/handlers/auth.rs
    @@ -5,4 +5,4 @@
     //! Login verifies against the legacy `FOSUser` table's password hash
    -//! (FOSUserBundle's default: sha512 over `password{salt}` iterated 5000
    -//! times, base64-encoded), then looks up the matching `PartKeeprUser`
    +//! (PartKeepr's sha512 over `password{salt}`, base64-encoded, or the
    +//! 5000-iteration variant used by earlier partkeepr-ng), then looks up the matching `PartKeeprUser`
     //! row by username — the legacy app-table that all the `user_id` FK
    @@ -141,8 +141,10 @@
     ///
    -/// FOSUserBundle's default `MessageDigestPasswordEncoder`:
    +/// PartKeepr configures Symfony's `MessageDigestPasswordEncoder` with
    +/// one iteration. Also accept the 5000-iteration Symfony default used by
    +/// earlier partkeepr-ng password resets. Hashes do not encode the count.
     ///
     /// ```text
    -/// salted = password + "{" + salt + "}"
    +/// salted = password + "{" + salt + "}" (or password when salt is empty)
     /// digest = sha512(salted)
    -/// for i in 1..5000:           # 4999 more iterations, 5000 total
    +/// for i in 1..iterations:
     ///     digest = sha512(digest || salted)
    @@ -153,5 +155,11 @@
     pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
    -    let salted = format!("{password}{{{salt}}}");
    +    let salted = if salt.is_empty() {
    +        password.to_owned()
    +    } else {
    +        format!("{password}{{{salt}}}")
    +    };
         let salted_bytes = salted.as_bytes();
         let mut digest = Sha512::digest(salted_bytes).to_vec();
    +    let legacy = base64::engine::general_purpose::STANDARD.encode(&digest);
    +    let legacy_matches = legacy.as_bytes().ct_eq(stored_hash_b64.as_bytes());
         for _ in 1..5000 {
    @@ -163,3 +171,30 @@
         let computed = base64::engine::general_purpose::STANDARD.encode(&digest);
    -    bool::from(computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
    +    bool::from(legacy_matches | computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
    +}
    +
    +#[cfg(test)]
    +mod password_tests {
    +    use super::verify_fos_password;
    +
    +    #[test]
    +    fn accepts_original_symfony_encoder_fixtures() {
    +        // Generated by the encoder in the original PartKeepr container,
    +        // using public fixture inputs, for both supported iteration counts.
    +        for (salt, hash) in [
    +            ("fixture-salt", "ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ=="),
    +            ("", "56Pet5rfF3IeVtwi7PY/i/RYiPiK3LqjvHoKZxDlneP760AzH47ELwwL7DA3dlezZ7nLYA8ET6scqvGGfms9sw=="),
    +            ("fixture-salt", "yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw=="),
    +            ("", "t6IMG9m1WloOJoNUlEHdxZncGEmOgYKjuZ2Cc132ouOL3jVIwlMBJ0h1Q48SzCSBR81PvCMFzI8Y0kwS/v0dDA=="),
    +        ] {
    +            assert!(verify_fos_password("migration-test-password", salt, hash));
    +            assert!(!verify_fos_password("incorrect-password", salt, hash));
    +        }
    +    }
    +
    +    #[test]
    +    fn rejects_invalid_hashes() {
    +        for hash in ["", "invalid", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="] {
    +            assert!(!verify_fos_password("migration-test-password", "fixture-salt", hash));
    +        }
    +    }
     }

## Activity

    $ sed -n '194,230p' /tmp/partkeepr-prod.sql; sed -n '677,705p' /tmp/partkeepr-prod.sql; rg -n '5000|5000-iter' README.md docs/migrating-from-partkeepr-1.4.0.md backend/src/bin/reset-password.rs; git diff --stat
    CREATE TABLE `FOSUser` (
      `id` int(11) NOT NULL AUTO_INCREMENT,
      `username` varchar(255) NOT NULL,
      `username_canonical` varchar(255) NOT NULL,
      `enabled` tinyint(1) NOT NULL,
      `salt` varchar(255) NOT NULL,
      `password` varchar(255) NOT NULL,
      `last_login` datetime DEFAULT NULL,
      `locked` tinyint(1) NOT NULL,
      `expired` tinyint(1) NOT NULL,
      `expires_at` datetime DEFAULT NULL,
      `confirmation_token` varchar(255) DEFAULT NULL,
      `password_requested_at` datetime DEFAULT NULL,
      `roles` longtext NOT NULL COMMENT '(DC2Type:array)',
      `credentials_expired` tinyint(1) NOT NULL,
      `credentials_expire_at` datetime DEFAULT NULL,
      `email_canonical` varchar(255) DEFAULT NULL,
      `email` varchar(255) DEFAULT NULL,
      PRIMARY KEY (`id`),
      UNIQUE KEY `UNIQ_37EF34F492FC23A8` (`username_canonical`)
    ) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
    /*!40101 SET character_set_client = @saved_cs_client */;
    
    --
    -- Dumping data for table `FOSUser`
    --
    
    LOCK TABLES `FOSUser` WRITE;
    /*!40000 ALTER TABLE `FOSUser` DISABLE KEYS */;
    INSERT INTO `FOSUser` VALUES (1,'alan','alan',1,'mm9p80yje80woowgswwso0gg8sk0oo4','X9CdnwCDZcRMK9DKSIs6qHOVc9nejuv6R13J4GFvZuH1EqcGhpuaIH6F4TlFA2u4xV0iuxYh1VC9Lkchh/1ZLw==',NULL,0,0,NULL,NULL,NULL,'a:1:{i:0;s:16:\"ROLE_SUPER_ADMIN\";}',0,NULL,'duncan.alan@me.com','duncan.alan@me.com');
    /*!40000 ALTER TABLE `FOSUser` ENABLE KEYS */;
    UNLOCK TABLES;
    
    --
    -- Table structure for table `Footprint`
    --
    
    CREATE TABLE `PartKeeprUser` (
      `id` int(11) NOT NULL AUTO_INCREMENT,
      `provider_id` int(11) DEFAULT NULL,
      `username` varchar(50) NOT NULL,
      `password` varchar(32) DEFAULT NULL,
      `email` varchar(255) DEFAULT NULL,
      `admin` tinyint(1) NOT NULL,
      `legacy` tinyint(1) NOT NULL,
      `lastSeen` datetime DEFAULT NULL,
      `active` tinyint(1) NOT NULL,
      `protected` tinyint(1) NOT NULL,
      PRIMARY KEY (`id`),
      UNIQUE KEY `username_provider` (`username`,`provider_id`),
      KEY `IDX_7572D706A53A8AA` (`provider_id`),
      CONSTRAINT `FK_7572D706A53A8AA` FOREIGN KEY (`provider_id`) REFERENCES `UserProvider` (`id`)
    ) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
    /*!40101 SET character_set_client = @saved_cs_client */;
    
    --
    -- Dumping data for table `PartKeeprUser`
    --
    
    LOCK TABLES `PartKeeprUser` WRITE;
    /*!40000 ALTER TABLE `PartKeeprUser` DISABLE KEYS */;
    INSERT INTO `PartKeeprUser` VALUES (1,1,'alan',NULL,NULL,0,0,NULL,1,0);
    /*!40000 ALTER TABLE `PartKeeprUser` ENABLE KEYS */;
    UNLOCK TABLES;
    
    --
    backend/src/bin/reset-password.rs:7://! (sha512 over `password{salt}` iterated 5000×, base64). Looks up
    backend/src/bin/reset-password.rs:144:    for _ in 1..5000 {
    docs/migrating-from-partkeepr-1.4.0.md:23:- `FOSUser` password hashes (sha512 + 5000 iter + per-user salt).
    README.md:41:  (sha512 + 5000-iter + per-user salt), so existing user accounts log
     backend/src/bin/reset-password.rs |  6 ++++-
     backend/src/handlers/auth.rs      | 49 +++++++++++++++++++++++++++++++++------
     2 files changed, 47 insertions(+), 8 deletions(-)
    ✓ • 0ms

## Activity

    $ cargo +stable test --locked --release --bin partkeepr-api password_tests
       Compiling partkeepr-api v0.1.13 (/home/alan/partkeepr-ng/backend)
        Finished `release` profile [optimized] target(s) in 8.67s
         Running unittests src/main.rs (target/release/deps/partkeepr_api-bf4fc9b8b712b94c)
    
    running 2 tests
    test handlers::auth::password_tests::rejects_invalid_hashes ... ok
    test handlers::auth::password_tests::accepts_original_symfony_encoder_fixtures ... ok
    
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
    
    ✓ • 8.60s

## Activity

    $ cat > local-install/check-password-login.py <<'PY'
        import subprocess,uuid,json,urllib.request,urllib.error,http.cookiejar
        name='migration_check_'+uuid.uuid4().hex[:12]
        base=['docker','compose','-f','local-install/compose.yaml','exec','-T','db','sh','-c','exec mariadb -N -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"']
        def sql(q): return subprocess.run(base,input=q,text=True,capture_output=True,check=True).stdout
        hashes=[('original single iteration','ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ=='),('5000 iterations','yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw==')]
        try:
         sql(f"INSERT INTO FOSUser(username,username_canonical,enabled,salt,password,locked,expired,roles,credentials_expired) VALUES ('{name}','{name}',1,'fixture-salt','{hashes[0][1]}',0,0,'a:0:{{}}',0); INSERT INTO PartKeeprUser(username,admin,legacy,active,protected) VALUES ('{name}',0,0,1,0);")
         for label,hash in hashes:
          sql(f"UPDATE FOSUser SET password='{hash}' WHERE username='{name}';")
          opener=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()),urllib.request.ProxyHandler({}))
          for password,expected in [('incorrect-password',401),('migration-test-password',200)]:
           req=urllib.request.Request('http://127.0.0.1:8082/api/login',data=json.dumps({'username':name,'password':password}).encode(),headers={'Content-Type':'application/json'})
           try:
            with opener.open(req) as r: status=r.status;r.read()
           except urllib.error.HTTPError as e: status=e.code
           assert status==expected,(label,status,expected)
          with opener.open('http://127.0.0.1:8082/api/me') as r:
           assert json.load(r)['username']==name
          print(label+': wrong password rejected, correct password accepted, session works')
        finally:
         sql(f"DELETE FROM PartKeeprUser WHERE username='{name}'; DELETE FROM FOSUser WHERE username='{name}';")
         print('Temporary test account removed; existing accounts unchanged.')
        PY
    ✓ • 68ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('README.md');s=p.read_text().replace('(sha512 + 5000-iter + per-user salt)', '(SHA-512 + per-user salt; original single-iteration and 5000-iteration hashes)');p.write_text(s)
        p=Path('docs/migrating-from-partkeepr-1.4.0.md');s=p.read_text().replace('`FOSUser` password hashes (sha512 + 5000 iter + per-user salt).','`FOSUser` password hashes (SHA-512 + per-user salt; original\n  single-iteration and 5000-iteration hashes are both accepted).');p.write_text(s)
        p=Path('backend/src/bin/reset-password.rs');s=p.read_text().replace('using\n//! the same FOSUserBundle scheme the login handler verifies against','using\n//! the 5000-iteration FOSUserBundle variant supported by the login handler').replace('/// Mirrors `verify_fos_password` in `handlers/auth.rs`. Keep them in\n/// sync; if either is changed, both must change.','/// The verifier in `handlers/auth.rs` accepts this variant as well as\n/// original PartKeepr single-iteration hashes. Empty salts are omitted.');p.write_text(s)
        PY
        cargo +stable build --locked --release --bin partkeepr-api
       Compiling partkeepr-api v0.1.13 (/home/alan/partkeepr-ng/backend)
        Finished `release` profile [optimized] target(s) in 24.97s
    ✓ • 24.97s

## Activity

    $ git diff --check; git diff --stat
     README.md                              |  2 +-
     backend/src/bin/reset-password.rs      | 12 ++++++---
     backend/src/handlers/auth.rs           | 49 +++++++++++++++++++++++++++++-----
     docs/migrating-from-partkeepr-1.4.0.md |  3 ++-
     4 files changed, 53 insertions(+), 13 deletions(-)
    ✓ • 0ms

## Activity

    $ cp target/release/partkeepr-api local-install/bin/partkeepr-api
    ✓ • 0ms

## Assistant

The regression tests pass for both hashing formats, including rejection of incorrect passwords. I’m deploying the fix now, then I’ll check the login endpoint with a temporary test account. Your `alan` account and password remain unchanged.

## Activity

    $ docker compose -f local-install/compose.yaml up -d --build app
    #0 building with "default" instance using docker driver
    
    #1 [app internal] load build definition from Dockerfile
    #1 transferring dockerfile: 273B done
    #1 DONE 0.0s
    
    #2 [app internal] load metadata for docker.io/library/debian:bookworm-slim
    #2 DONE 0.3s
    
    #3 [app internal] load .dockerignore
    #3 transferring context: 79B done
    #3 DONE 0.0s
    
    #4 [app 1/3] FROM docker.io/library/debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587
    #4 resolve docker.io/library/debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587 0.0s done
    #4 DONE 0.0s
    
    #5 [app internal] load build context
    #5 transferring context: 19.71MB 0.1s done
    #5 DONE 0.1s
    
    #6 [app 2/3] RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
    #6 CACHED
    
    #7 [app 3/3] COPY bin/partkeepr-api /usr/local/bin/partkeepr-api
    #7 DONE 0.2s
    
    #8 [app] exporting to image
    #8 exporting layers
    #8 exporting layers 0.8s done
    #8 exporting manifest sha256:8ffa7b810b55f1efee3e95adcc328315f3034b532efaee736df01e7ef0d240c8 done
    #8 exporting config sha256:577df3a6185c82c25903416edb2f24e88cc29aede1b90b803136a538d58e8527 done
    #8 exporting attestation manifest sha256:6bface4bfe0e451336c54b42c42f1de45189147f7236bdf0ccb9bd8f731b2b26 done
    #8 exporting manifest list sha256:04b6db0401040213063e2f06a4ae3ccf65a54847169de4bf1ae70c33606605d6 done
    #8 naming to docker.io/library/partkeepr-ng-app:latest done
    #8 unpacking to docker.io/library/partkeepr-ng-app:latest 0.1s done
    #8 DONE 0.9s
     Container partkeepr-ng-db-1  Running
     Container partkeepr-ng-app-1  Recreate
     Container partkeepr-ng-app-1  Recreated
     Container partkeepr-ng-db-1  Waiting
     Container partkeepr-ng-db-1  Healthy
     Container partkeepr-ng-app-1  Starting
     Container partkeepr-ng-app-1  Started
    ✓ • 13.01s

## Activity

    $ python3 local-install/check-password-login.py; python3 local-install/verify.py
    original single iteration: wrong password rejected, correct password accepted, session works
    5000 iterations: wrong password rejected, correct password accepted, session works
    Temporary test account removed; existing accounts unchanged.
    {'path': '/health', 'status': 200, 'bytes': 2}
    {'path': '/', 'status': 200, 'bytes': 1090}
    {'path': '/api/me (unauthenticated)', 'status': 401}
    {'path': '/api/me', 'status': 200, 'bytes': 48}
    {'path': '/api/version', 'status': 200, 'bytes': 20}
    {'path': '/api/parts', 'status': 200, 'bytes': 29574}
    {'path': '/api/part_categories/tree', 'status': 200, 'bytes': 27590}
    {'path': '/api/storage_tree', 'status': 200, 'bytes': 47090}
    {'path': '/api/footprint_tree', 'status': 200, 'bytes': 14752}
    {'path': '/api/projects', 'status': 200, 'bytes': 2}
    {'path': '/api/parts/1', 'status': 200, 'bytes': 1336}
    {'path': '/files/PartAttachment/1', 'status': 200, 'bytes': 324405}
    {'path': '/files/FootprintAttachment/1', 'status': 200, 'bytes': 138512}
    {'path': '/files/FootprintImage/1', 'status': 200, 'bytes': 23365}
    {'path': '/files/FootprintImage/1/thumb', 'status': 200, 'bytes': 10066}
    {'path': '/files/ManufacturerICLogo/1', 'status': 200, 'bytes': 2195}
    {'path': '/files/ManufacturerICLogo/1/thumb', 'status': 200, 'bytes': 3718}
    {'path': '/files/StorageLocationImage/1', 'status': 200, 'bytes': 3927785}
    {'path': '/files/StorageLocationImage/1/thumb', 'status': 200, 'bytes': 10860}
    ✓ • 1.75s

## Assistant

Fixed and deployed. Partkeepr-ng incorrectly assumed 5,000 password-hashing iterations; your original PartKeepr uses one.

Both formats now pass password-login and session tests, and incorrect passwords are rejected. Your password is unchanged.

Please try **`alan` with your original password** again at [http://10.0.0.1:8082/](http://10.0.0.1:8082/).

## User

Question - no action to take for implementation - but is it required to run the application in a Docker container, or can it run natively with systemd. Same for our MariaDB? Just wondering.

## Assistant

Docker isn’t required for either. **Partkeepr-ng can run natively under systemd, and MariaDB can run as a normal system service.** You can also mix them—native app with containerized MariaDB, for example.

The app is a compiled Rust executable plus the static `frontend/` directory. A systemd service would run that executable under a dedicated user, with configuration pointing to MariaDB, the frontend, and writable attachment storage. No Node.js or PHP runtime is needed.

MariaDB needs version **10.6 or newer**. Moving the current database to native MariaDB would involve dumping and restoring it, then updating the app’s connection URL.

I used Docker here to keep the migration isolated from your existing services and original PartKeepr installation. Native systemd services are a reasonable permanent setup too.

## User

I'm sorry, I really am trying to move away from the complexity of Docker containers in favour of natively-run web applications. Is it possible for you to re-do this in the way we just discussed? systemd managed?

## Assistant

I’ll move the app and its current database to native systemd services, preserving the password fix and imported files. I’ll first check the host’s existing database service so the migration doesn’t disrupt other applications.

## Activity

    $ git status --short; cat local-install/compose.yaml; cat /etc/os-release; command -v mariadbd mysqld; ls -ld /opt /var/lib/partkeepr-ng; rg --files -g AGENTS.md . /home/alan 2>/dev/null
     M README.md
     M backend/src/bin/reset-password.rs
     M backend/src/handlers/auth.rs
     M docs/migrating-from-partkeepr-1.4.0.md
    name: partkeepr-ng
    services:
      db:
        image: mariadb:10.11
        restart: unless-stopped
        env_file: database.env
        volumes:
          - database:/var/lib/mysql
        healthcheck:
          test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
          interval: 5s
          timeout: 5s
          retries: 30
      app:
        build:
          context: .
          dockerfile: Dockerfile
        restart: unless-stopped
        user: "1000:1000"
        env_file: app.env
        ports:
          - "127.0.0.1:8082:8080"
          - "10.0.0.1:8082:8080"
          - "192.168.5.109:8082:8080"
        volumes:
          - ./storage:/storage
          - ../frontend:/frontend:ro
        depends_on:
          db:
            condition: service_healthy
    volumes:
      database:
    PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
    NAME="Debian GNU/Linux"
    VERSION_ID="12"
    VERSION="12 (bookworm)"
    VERSION_CODENAME=bookworm
    ID=debian
    HOME_URL="https://www.debian.org/"
    SUPPORT_URL="https://www.debian.org/support"
    BUG_REPORT_URL="https://bugs.debian.org/"
    /usr/sbin/mysqld
    ls: cannot access '/var/lib/partkeepr-ng': No such file or directory
    drwxr-xr-x 11 nobody nogroup 4096 Oct 10 02:00 /opt
    /home/alan/obsidian-livesync/AGENTS.md
    /home/alan/translation-server/modules/translators/AGENTS.md
    ✗ (2) • 0ms

## Activity

    $ systemctl status mysql mariadb --no-pager; systemctl is-enabled docker; dpkg-query -W 'mysql*' 'mariadb*' 2>/dev/null; ss -ltn '( sport = :3306 or sport = :8082 )'; sudo -n true
    Unit mariadb.service could not be found.
    ● mysql.service - MySQL Community Server
         Loaded: loaded (/lib/systemd/system/mysql.service; enabled; preset: enabled)
         Active: active (running) since Thu 2026-09-03 16:42:35 EDT; 1 month 12 days ago
           Docs: man:mysqld(8)
                 http://dev.mysql.com/doc/refman/en/using-systemd.html
       Main PID: 3700 (mysqld)
         Status: "Server is operational"
          Tasks: 39 (limit: 38303)
         Memory: 205.2M
            CPU: 6h 58min 45.905s
         CGroup: /system.slice/mysql.service
                 └─3700 /usr/sbin/mysqld
    
    Warning: some journal files were not opened due to insufficient permissions.
    enabled
    mariadb-client-10.0
    mariadb-client-10.1
    mariadb-client-10.2
    mariadb-client-10.3
    mariadb-client-5.5
    mariadb-client-core-10.0
    mariadb-client-core-10.1
    mariadb-client-core-10.2
    mariadb-client-core-10.3
    mariadb-client-core-5.5
    mariadb-server-10.0
    mariadb-server-10.1
    mariadb-server-10.2
    mariadb-server-10.3
    mariadb-server-5.5
    mariadb-server-core-10.0
    mariadb-server-core-10.1
    mariadb-server-core-10.2
    mariadb-server-core-10.3
    mariadb-server-core-5.5
    mysql
    mysql-apt-config    0.8.30-1
    mysql-client    8.0.35-1ubuntu23.04
    mysql-client-5.0
    mysql-client-5.1
    mysql-client-5.5
    mysql-client-5.6
    mysql-client-5.7
    mysql-client-8.0
    mysql-client-core-5.0
    mysql-client-core-5.1
    mysql-client-core-5.5
    mysql-client-core-5.6
    mysql-client-core-5.7
    mysql-client-core-8.0
    mysql-cluster-commercial-client
    mysql-cluster-commercial-server
    mysql-cluster-community-client
    mysql-cluster-community-server
    mysql-commercial-client
    mysql-commercial-client-core
    mysql-commercial-client-plugins
    mysql-commercial-server
    mysql-commercial-server-core
    mysql-common    8.0.35-1ubuntu23.04
    mysql-community-client    8.0.35-1ubuntu23.04
    mysql-community-client-core    8.0.35-1ubuntu23.04
    mysql-community-client-plugins    8.0.35-1ubuntu23.04
    mysql-community-server    8.0.35-1ubuntu23.04
    mysql-community-server-core    8.0.35-1ubuntu23.04
    mysql-server    8.0.35-1ubuntu23.04
    mysql-server-5.0
    mysql-server-5.1
    mysql-server-5.5
    mysql-server-5.6
    mysql-server-5.7
    mysql-server-8.0
    mysql-server-core-5.0
    mysql-server-core-5.1
    mysql-server-core-5.5
    mysql-server-core-5.6
    mysql-server-core-5.7
    mysql-server-core-8.0
    State  Recv-Q Send-Q Local Address:Port Peer Address:PortProcess
    LISTEN 0      4096        10.0.0.1:8082      0.0.0.0:*
    LISTEN 0      4096       127.0.0.1:8082      0.0.0.0:*
    LISTEN 0      4096   192.168.5.109:8082      0.0.0.0:*
    LISTEN 0      151                *:3306            *:*
    sudo: a password is required
    ✗ (1) • 93ms

## Activity

    $ systemctl --user is-system-running; loginctl show-user alan -p Linger -p State; apt-cache policy mariadb-server mariadb-server-core mariadb-client-core liburing2 libaio1; command -v dpkg-deb; ls -l /usr/lib/x86_64-linux-gnu/liburing* /usr/lib/x86_64-linux-gnu/libaio* 2>/dev/null; ls -ld /run/user/1000; cat /proc/sys/kernel/unprivileged_userns_clone
    running
    State=active
    Linger=no
    mariadb-server:
      Installed: (none)
      Candidate: 1:10.11.19-0+deb12u1
      Version table:
         1:10.11.19-0+deb12u1 500
            500 http://security.debian.org/debian-security bookworm-security/main amd64 Packages
         1:10.11.18-0+deb12u1 500
            500 http://deb.debian.org/debian bookworm/main amd64 Packages
    mariadb-server-core:
      Installed: (none)
      Candidate: 1:10.11.19-0+deb12u1
      Version table:
         1:10.11.19-0+deb12u1 500
            500 http://security.debian.org/debian-security bookworm-security/main amd64 Packages
         1:10.11.18-0+deb12u1 500
            500 http://deb.debian.org/debian bookworm/main amd64 Packages
    mariadb-client-core:
      Installed: (none)
      Candidate: 1:10.11.19-0+deb12u1
      Version table:
         1:10.11.19-0+deb12u1 500
            500 http://security.debian.org/debian-security bookworm-security/main amd64 Packages
         1:10.11.18-0+deb12u1 500
            500 http://deb.debian.org/debian bookworm/main amd64 Packages
    liburing2:
      Installed: (none)
      Candidate: 2.3-3
      Version table:
         2.3-3 500
            500 http://deb.debian.org/debian bookworm/main amd64 Packages
    libaio1:
      Installed: 0.3.113-4
      Candidate: 0.3.113-4
      Version table:
     *** 0.3.113-4 500
            500 http://deb.debian.org/debian bookworm/main amd64 Packages
            100 /var/lib/dpkg/status
    /usr/bin/dpkg-deb
    zsh:1: no matches found: /usr/lib/x86_64-linux-gnu/liburing*
    drwx------ 15 alan alan 520 Oct 11 09:34 /run/user/1000
    1
    ✓ • 0ms

## Activity

    $ apt-get -s install mariadb-server mariadb-client; apt-cache show mariadb-server-core | head -24; apt-cache show mariadb-server | head -25; rg -n 'REGEXP|RETURNING|JSON_|MariaDB' backend/src | head -35
    NOTE: This is only a simulation!
          apt-get needs root privileges for real execution.
          Keep also in mind that locking is deactivated,
          so don't depend on the relevance to the real current situation!
    Reading package lists...
    Building dependency tree...
    Reading state information...
    The following packages were automatically installed and are no longer required:
      libmecab2 libslirp0 mecab-ipadic mecab-ipadic-utf8 mecab-utils
      mysql-community-client-plugins slirp4netns
    Use 'apt autoremove' to remove them.
    The following additional packages will be installed:
      galera-4 gawk libconfig-inifiles-perl libdbd-mariadb-perl libdbi-perl
      libhtml-template-perl libmariadb3 libperl5.36 libsigsegv2
      libterm-readkey-perl liburing2 mariadb-client-core mariadb-common
      mariadb-plugin-provider-bzip2 mariadb-plugin-provider-lz4
      mariadb-plugin-provider-lzma mariadb-plugin-provider-lzo
      mariadb-plugin-provider-snappy mariadb-server-core perl perl-base
      perl-modules-5.36 pv
    Suggested packages:
      gawk-doc libmldbm-perl libnet-daemon-perl libsql-statement-perl
      libipc-sharedcache-perl mariadb-test perl-doc libterm-readline-gnu-perl
      | libterm-readline-perl-perl libtap-harness-archive-perl doc-base
    The following packages will be REMOVED:
      mysql-client mysql-community-client mysql-community-client-core
      mysql-community-server mysql-community-server-core mysql-server
    The following NEW packages will be installed:
      galera-4 gawk libconfig-inifiles-perl libdbd-mariadb-perl libdbi-perl
      libhtml-template-perl libmariadb3 libsigsegv2 libterm-readkey-perl liburing2
      mariadb-client mariadb-client-core mariadb-common
      mariadb-plugin-provider-bzip2 mariadb-plugin-provider-lz4
      mariadb-plugin-provider-lzma mariadb-plugin-provider-lzo
      mariadb-plugin-provider-snappy mariadb-server mariadb-server-core pv
    The following packages will be upgraded:
      libperl5.36 perl perl-base perl-modules-5.36
    4 upgraded, 21 newly installed, 6 to remove and 553 not upgraded.
    Inst libperl5.36 [5.36.0-7+deb12u1] (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64]) [perl:amd64 ]
    Inst perl [5.36.0-7+deb12u1] (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64]) []
    Inst perl-base [5.36.0-7+deb12u1] (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64]) []
    Conf perl-base (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64]) []
    Inst perl-modules-5.36 [5.36.0-7+deb12u1] (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [all])
    Remv mysql-server [8.0.35-1ubuntu23.04]
    Remv mysql-community-server [8.0.35-1ubuntu23.04]
    Remv mysql-client [8.0.35-1ubuntu23.04]
    Remv mysql-community-client [8.0.35-1ubuntu23.04]
    Remv mysql-community-client-core [8.0.35-1ubuntu23.04]
    Remv mysql-community-server-core [8.0.35-1ubuntu23.04]
    Inst libsigsegv2 (2.14-1 Debian:12.15/oldstable [amd64])
    Conf libsigsegv2 (2.14-1 Debian:12.15/oldstable [amd64])
    Inst gawk (1:5.2.1-2 Debian:12.15/oldstable [amd64])
    Inst mariadb-common (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [all])
    Inst galera-4 (26.4.23-0+deb12u1 Debian:12.15/oldstable [amd64])
    Inst libdbi-perl (1.643-4+deb12u3 Debian-Security:12/oldstable-security [amd64])
    Inst libconfig-inifiles-perl (3.000003-2+deb12u1 Debian:12.15/oldstable, Debian-Security:12/oldstable-security [all])
    Inst libmariadb3 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-client-core (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-client (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst liburing2 (2.3-3 Debian:12.15/oldstable [amd64])
    Inst mariadb-server-core (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-common (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [all])
    Inst mariadb-server (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst libdbd-mariadb-perl (1.22-1+b1 Debian:12.15/oldstable [amd64])
    Inst libhtml-template-perl (2.97-2 Debian:12.15/oldstable [all])
    Inst libterm-readkey-perl (2.38-2+b1 Debian:12.15/oldstable [amd64])
    Inst mariadb-plugin-provider-bzip2 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-plugin-provider-lz4 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-plugin-provider-lzma (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-plugin-provider-lzo (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst mariadb-plugin-provider-snappy (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Inst pv (1.6.20-1 Debian:12.15/oldstable [amd64])
    Conf libperl5.36 (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64])
    Conf perl (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [amd64])
    Conf perl-modules-5.36 (5.36.0-7+deb12u4 Debian-Security:12/oldstable-security [all])
    Conf gawk (1:5.2.1-2 Debian:12.15/oldstable [amd64])
    Conf galera-4 (26.4.23-0+deb12u1 Debian:12.15/oldstable [amd64])
    Conf libdbi-perl (1.643-4+deb12u3 Debian-Security:12/oldstable-security [amd64])
    Conf libconfig-inifiles-perl (3.000003-2+deb12u1 Debian:12.15/oldstable, Debian-Security:12/oldstable-security [all])
    Conf libmariadb3 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-client-core (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-client (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf liburing2 (2.3-3 Debian:12.15/oldstable [amd64])
    Conf mariadb-server-core (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-server (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf libdbd-mariadb-perl (1.22-1+b1 Debian:12.15/oldstable [amd64])
    Conf libhtml-template-perl (2.97-2 Debian:12.15/oldstable [all])
    Conf libterm-readkey-perl (2.38-2+b1 Debian:12.15/oldstable [amd64])
    Conf mariadb-plugin-provider-bzip2 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-plugin-provider-lz4 (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-plugin-provider-lzma (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-plugin-provider-lzo (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf mariadb-plugin-provider-snappy (1:10.11.19-0+deb12u1 Debian-Security:12/oldstable-security [amd64])
    Conf pv (1.6.20-1 Debian:12.15/oldstable [amd64])
    Package: mariadb-server-core
    Source: mariadb
    Version: 1:10.11.19-0+deb12u1
    Installed-Size: 45336
    Maintainer: Debian MySQL Maintainers <pkg-mysql-maint@lists.alioth.debian.org>
    Architecture: amd64
    Replaces: mariadb-client-10.1, mariadb-server (<< 1:10.11.19-0+deb12u1), mariadb-server-10.0, mariadb-server-10.1, mariadb-server-10.2, mariadb-server-10.3, mariadb-server-10.4, mariadb-server-10.5 (<< 1:10.11), mariadb-server-10.6, mariadb-server-core-10.0, mariadb-server-core-10.1, mariadb-server-core-10.2, mariadb-server-core-10.3, mariadb-server-core-10.4, mariadb-server-core-10.5, mariadb-server-core-10.6, mariadb-server-core-5.5, mysql-client-5.5, mysql-client-5.6, mysql-client-5.7, mysql-client-8.0, mysql-server-core-5.5, mysql-server-core-5.6, mysql-server-core-5.7, mysql-server-core-8.0, percona-server-server-5.6, percona-xtradb-cluster-server-5.6, percona-xtradb-cluster-server-5.7, virtual-mysql-server-core
    Provides: virtual-mysql-server-core
    Depends: mariadb-common (>= 1:10.11.19-0+deb12u1), libaio1 (>= 0.3.93), libc6 (>= 2.34), libcrypt1 (>= 1:4.1.0), libgcc-s1 (>= 3.0), libnuma1 (>= 2.0.11), libpcre2-8-0 (>= 10.22), libssl3 (>= 3.0.0), libstdc++6 (>= 12), libsystemd0 (>= 227), liburing2 (>= 2.3), zlib1g (>= 1:1.2.0)
    Conflicts: mariadb-galera-server-5.5, mysql-server-5.5, mysql-server-5.6, mysql-server-5.7, mysql-server-8.0, virtual-mysql-server-core
    Breaks: mariadb-client-10.1, mariadb-server-10.0, mariadb-server-10.1, mariadb-server-10.2, mariadb-server-10.3, mariadb-server-10.4, mariadb-server-10.5 (<< 1:10.11), mariadb-server-core-10.0, mariadb-server-core-10.1, mariadb-server-core-10.2, mariadb-server-core-10.3, mariadb-server-core-10.4, mariadb-server-core-10.5, mariadb-server-core-10.6, mariadb-server-core-5.5, mysql-client-5.5, mysql-client-5.6, mysql-server-core-5.5, mysql-server-core-5.6, mysql-server-core-5.7, mysql-server-core-8.0, percona-server-server-5.6, percona-xtradb-cluster-server-5.6, percona-xtradb-cluster-server-5.7
    Description-en: MariaDB database core server files
     MariaDB is a fast, stable and true multi-user, multi-threaded SQL database
     server. SQL (Structured Query Language) is the most popular database query
     language in the world. The main goals of MariaDB are speed, robustness and
     ease of use.
     .
     This package includes the core server files, as used by Akonadi.
    Description-md5: a01f76260dfbedfdfbfcd8ff440b8a4c
    Homepage: https://mariadb.org/
    Section: database
    Priority: optional
    Filename: pool/updates/main/m/mariadb/mariadb-server-core_10.11.19-0+deb12u1_amd64.deb
    Size: 7342456
    E: Write error - write (32: Broken pipe)
    Package: mariadb-server
    Source: mariadb
    Version: 1:10.11.19-0+deb12u1
    Installed-Size: 53815
    Maintainer: Debian MySQL Maintainers <pkg-mysql-maint@lists.alioth.debian.org>
    Architecture: amd64
    Replaces: handlersocket-mysql-5.5, mariadb-galera-server, mariadb-galera-server-10.0, mariadb-galera-server-5.5, mariadb-server-10.0, mariadb-server-10.1, mariadb-server-10.2, mariadb-server-10.3, mariadb-server-10.4, mariadb-server-10.5 (<< 1:10.11), mariadb-server-10.6, mariadb-server-5.5, mariadb-tokudb-engine-10.0, mariadb-tokudb-engine-10.1, mariadb-tokudb-engine-5.5, mysql-client-5.5, mysql-client-5.7, mysql-client-core-8.0, mysql-server-5.5, mysql-server-5.6, mysql-server-5.7, mysql-server-8.0, percona-server-server-5.6, percona-xtradb-cluster-server-5.6, percona-xtradb-cluster-server-5.7, virtual-mysql-server
    Provides: virtual-mysql-server
    Depends: galera-4 (>= 26.4), gawk, iproute2, libdbi-perl, lsof, mariadb-client (>= 1:10.11.19-0+deb12u1), mariadb-server-core (>= 1:10.11.19-0+deb12u1), passwd, perl (>= 5.6), procps, psmisc, rsync, socat, debconf (>= 0.5) | debconf-2.0, libc6 (>= 2.34), libgcc-s1 (>= 3.0), libpam0g (>= 0.99.7.1), libssl3 (>= 3.0.0), libstdc++6 (>= 11), zlib1g (>= 1:1.1.4)
    Suggests: mariadb-test, netcat-openbsd
    Conflicts: handlersocket-mysql-5.5, mariadb-tokudb-engine-10.0, mariadb-tokudb-engine-10.1, mariadb-tokudb-engine-5.5, mysql-server-core-5.5, mysql-server-core-5.6, mysql-server-core-5.7, mysql-server-core-8.0, percona-server-server-5.6, percona-xtradb-cluster-server-5.6, percona-xtradb-cluster-server-5.7, virtual-mysql-server
    Breaks: cqrlog (<< 1.9.0-5~), galera-3 (<< 26.4), handlersocket-mysql-5.5, mariadb-galera-server, mariadb-galera-server-10.0, mariadb-galera-server-5.5, mariadb-server-10.0, mariadb-server-10.1, mariadb-server-10.2, mariadb-server-10.3, mariadb-server-10.4, mariadb-server-10.5 (<< 1:10.11), mariadb-server-10.6, mariadb-server-5.5, mariadb-tokudb-engine-10.0, mariadb-tokudb-engine-10.1, mariadb-tokudb-engine-5.5, mysql-client-5.5, mysql-client-5.7, mysql-client-core-8.0, mysql-server-5.5, mysql-server-5.6, mysql-server-5.7, mysql-server-8.0
    Description-en: MariaDB database server binaries
     MariaDB is a fast, stable and true multi-user, multi-threaded SQL database
     server. SQL (Structured Query Language) is the most popular database query
     language in the world. The main goals of MariaDB are speed, robustness and
     ease of use.
     .
     This package includes the server binaries.
    Description-md5: ae76dbcde4bf3e79b8228e6dad967834
    Homepage: https://mariadb.org/
    Pre-Depends: adduser (>= 3.40), debconf, mariadb-common (>= 1:10.11.19-0+deb12u1)
    Recommends: libhtml-template-perl, mariadb-plugin-provider-bzip2, mariadb-plugin-provider-lz4, mariadb-plugin-provider-lzma, mariadb-plugin-provider-lzo, mariadb-plugin-provider-snappy, pv
    Section: database
    Priority: optional
    E: Write error - write (32: Broken pipe)
    backend/src/handlers/lookups.rs:1014:/// Map MySQL/MariaDB error 1062 (duplicate-key on UNIQUE) to a friendly
    backend/src/handlers/parts.rs:297:        // Reject MariaDB's zero-datetime explicitly — comparing it to
    backend/src/handlers/parts.rs:1288:    // MySQL/MariaDB FK constraint violation is error code 1452 (insert/update)
    backend/src/handlers/grid_presets.rs:161:/// Map MariaDB error 1062 (duplicate-key on `name_grid_unique`) →
    ✓ • 848ms

## Assistant

The host already runs MySQL for other applications. Debian’s standard MariaDB installation would remove those MySQL packages, so I’ll keep this MariaDB instance separate, on a local-only port.

I’m checking whether systemd can run both services under your user account at boot. That would let us complete the migration without replacing the existing database server or requiring sudo access.

## Activity

    $ loginctl enable-linger alan; loginctl show-user alan -p Linger; mkdir -p local-install/native/packages; apt-get download mariadb-server-core mariadb-server mariadb-client-core mariadb-common liburing2
    Linger=yes
    Get:1 http://security.debian.org/debian-security bookworm-security/main amd64 mariadb-client-core amd64 1:10.11.19-0+deb12u1 [870 kB]
    Get:2 http://deb.debian.org/debian bookworm/main amd64 liburing2 amd64 2.3-3 [12.6 kB]
    Get:3 http://security.debian.org/debian-security bookworm-security/main amd64 mariadb-common all 1:10.11.19-0+deb12u1 [27.6 kB]
    Get:4 http://security.debian.org/debian-security bookworm-security/main amd64 mariadb-server amd64 1:10.11.19-0+deb12u1 [3666 kB]
    Get:5 http://security.debian.org/debian-security bookworm-security/main amd64 mariadb-server-core amd64 1:10.11.19-0+deb12u1 [7342 kB]
    Fetched 11.9 MB in 1s (9924 kB/s)
    ✓ • 1.43s

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import subprocess
        root=Path('local-install/native');dest=root/'mariadb'
        dest.mkdir(exist_ok=True)
        for p in Path('.').glob('*.deb'):
         if p.name.startswith(('mariadb-','liburing2_')): p.rename(root/'packages'/p.name)
        for p in sorted((root/'packages').glob('*.deb')):
         subprocess.run(['dpkg-deb','-x',str(p),str(dest)],check=True)
        print('Extracted Debian binaries privately; no host packages installed or removed.')
        PY
        LD_LIBRARY_PATH=/home/alan/partkeepr-ng/local-install/native/mariadb/usr/lib/x86_64-linux-gnu ldd local-install/native/mariadb/usr/sbin/mariadbd
        sed -n '1,110p' local-install/native/mariadb/usr/bin/mariadb-install-db
    Extracted Debian binaries privately; no host packages installed or removed.
        linux-vdso.so.1 (0x00007ffd77be5000)
        libpcre2-8.so.0 => /lib/x86_64-linux-gnu/libpcre2-8.so.0 (0x00007fa683697000)
        libcrypt.so.1 => /lib/x86_64-linux-gnu/libcrypt.so.1 (0x00007fa68365b000)
        liburing.so.2 => /home/alan/partkeepr-ng/local-install/native/mariadb/usr/lib/x86_64-linux-gnu/liburing.so.2 (0x00007fa683654000)
        libaio.so.1 => /lib/x86_64-linux-gnu/libaio.so.1 (0x00007fa68364f000)
        libnuma.so.1 => /lib/x86_64-linux-gnu/libnuma.so.1 (0x00007fa683642000)
        libsystemd.so.0 => /lib/x86_64-linux-gnu/libsystemd.so.0 (0x00007fa681330000)
        libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x00007fa683621000)
        libssl.so.3 => /lib/x86_64-linux-gnu/libssl.so.3 (0x00007fa681287000)
        libcrypto.so.3 => /lib/x86_64-linux-gnu/libcrypto.so.3 (0x00007fa680e00000)
        libstdc++.so.6 => /lib/x86_64-linux-gnu/libstdc++.so.6 (0x00007fa680a00000)
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007fa680d21000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007fa6835ff000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007fa68081f000)
        /lib64/ld-linux-x86-64.so.2 (0x00007fa683750000)
        libcap.so.2 => /lib/x86_64-linux-gnu/libcap.so.2 (0x00007fa6835f3000)
        libgcrypt.so.20 => /lib/x86_64-linux-gnu/libgcrypt.so.20 (0x00007fa6806d8000)
        liblzma.so.5 => /lib/x86_64-linux-gnu/liblzma.so.5 (0x00007fa6835c4000)
        libzstd.so.1 => /lib/x86_64-linux-gnu/libzstd.so.1 (0x00007fa680c65000)
        liblz4.so.1 => /lib/x86_64-linux-gnu/liblz4.so.1 (0x00007fa680c3f000)
        libgpg-error.so.0 => /lib/x86_64-linux-gnu/libgpg-error.so.0 (0x00007fa6806b0000)
    #!/bin/sh
    # Copyright (c) 2000, 2013, Oracle and/or its affiliates.
    # Copyright (c) 2009, 2013, Monty Program Ab
    #
    # This program is free software; you can redistribute it and/or modify
    # it under the terms of the GNU General Public License as published by
    # the Free Software Foundation; version 2 of the License.
    #
    # This program is distributed in the hope that it will be useful,
    # but WITHOUT ANY WARRANTY; without even the implied warranty of
    # MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    # GNU General Public License for more details.
    #
    # You should have received a copy of the GNU General Public License
    # along with this program; if not, write to the Free Software
    # Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1335  USA
    
    # This scripts creates the MariaDB Server system tables
    #
    # All unrecognized arguments to this script are passed to mariadbd.
    
    basedir=""
    builddir=""
    ldata="/var/lib/mysql"
    langdir=""
    srcdir=""
    
    args=""
    defaults=""
    defaults_group_suffix=""
    mysqld_opt=""
    user=""
    group=""
    silent_startup="--silent-startup"
    log_error=""
    
    force=0
    in_rpm=0
    ip_only=0
    cross_bootstrap=0
    do_resolve=0
    auth_root_authentication_method=socket
    auth_root_socket_user=""
    skip_test_db=0
    extra_file=""
    
    dirname0=`dirname $0 2>/dev/null`
    dirname0=`dirname $dirname0 2>/dev/null`
    
    usage()
    {
      cat <<EOF
    Usage: $0 [OPTIONS]
      --auth-root-authentication-method=normal|socket
                           Chooses the authentication method for the created
                           initial root user. The historical behavior is 'normal'
                           to creates a root user that can login without password,
                           which can be insecure. The default behavior 'socket'
                           sets an invalid root password but allows the system root
                           user to login as MariaDB root without a password.
      --auth-root-socket-user=user
                           Used with --auth-root-authentication-method=socket. It
                           specifies the name of the second MariaDB root account,
                           as well as of the system account allowed to access it.
                           Defaults to the value of --user.
      --basedir=path       The path to the MariaDB installation directory.
      --builddir=path      If using --srcdir with out-of-directory builds, you
                           will need to set this to the location of the build
                           directory where built files reside.
      --cross-bootstrap    For internal use.  Used when building the MariaDB system
                           tables on a different host than the target.
      --datadir=path       The path to the MariaDB data directory.
      --no-defaults        Don't read default options from any option file.
      --defaults-extra-file=name
                           Read this file after the global files are read.
      --defaults-file=name Only read default options from the given file name.
      --defaults-group-suffix=name
                           In addition to the given groups, read also groups with
                           this suffix
      --force              Causes mysql_install_db to run even if DNS does not
                           work.  In that case, grant table entries that
                           normally use hostnames will use IP addresses.
      --help               Display this help and exit.
      --ldata=path         The path to the MariaDB data directory. Same as
                           --datadir.
      --rpm                For internal use.  This option is used by RPM files
                           during the MariaDB installation process.
      --skip-name-resolve  Use IP addresses rather than hostnames when creating
                           grant table entries.  This option can be useful if
                           your DNS does not work.
      --skip-test-db       Don't install a test database.
      --srcdir=path        The path to the MariaDB source directory.  This option
                           uses the compiled binaries and support files within the
                           source tree, useful for if you don't want to install
                           MariaDB yet and just want to create the system tables.
      --user=user_name     The login username to use for running mariadbd.  Files
                           and directories created by mariadbd will be owned by this
                           user.  You must be root to use this option.  By default
                           mariadbd runs using your current login name and files and
                           directories that it creates will be owned by you.
      --group=group_name   The login group to use for running mariadbd.  Files and
                           directories created by mariadbd will be owned by this
                           group. You must be root to use this option.  By default
                           mariadbd runs using your current group and files and
                           directories that it creates will be owned by you.
      --extra-file=file    Add user defined SQL file, to be executed following
                           regular database initialization.
    
    All other options are passed to the mariadbd program
    
    ✓ • 601ms

## Activity

    $ mkdir -p local-install/native/database local-install/native/backups; chmod 700 local-install/native/database local-install/native/backups; LD_LIBRARY_PATH=/home/alan/partkeepr-ng/local-install/native/mariadb/usr/lib/x86_64-linux-gnu local-install/native/mariadb/usr/bin/mariadb-install-db --no-defaults --basedir=/home/alan/partkeepr-ng/local-install/native/mariadb/usr --datadir=/home/alan/partkeepr-ng/local-install/native/database --auth-root-authentication-method=socket --auth-root-socket-user=alan --skip-test-db
    Installing MariaDB/MySQL system tables in '/home/alan/partkeepr-ng/local-install/native/database' ...
    2026-10-16  6:54:10 0 [Warning] failed to retrieve the MAC address
    2026-10-16  6:54:10 0 [Warning] WSREP: Failed to guess base node address. Set it explicitly via wsrep_node_address.
    2026-10-16  6:54:10 0 [Warning] WSREP: Failed to guess base node address. Set it explicitly via wsrep_node_address.
    2026-10-16  6:54:10 0 [Warning] WSREP: Guessing address for incoming client connections failed. Try setting wsrep_node_incoming_address explicitly.
    2026-10-16  6:54:10 0 [Warning] mariadbd: io_uring_queue_init() failed with EPERM: sysctl kernel.io_uring_disabled has the value 2, or 1 and the user of the process is not a member of sysctl kernel.io_uring_group. (see man 2 io_uring_setup).
    create_uring failed: falling back to libaio
    OK
    
    To start mariadbd at boot time you have to copy
    support-files/mariadb.service to the right place for your system
    
    
    Two all-privilege accounts were created.
    One is root@localhost, it has no password, but you need to
    be system 'root' user to connect. Use, for example, sudo mariadb
    The second is alan@localhost, it has no password either, but
    you need to be the system 'alan' user to connect.
    After connecting you can set the password, if you would need to be
    able to connect as any of these users with a password and without sudo
    
    See the MariaDB Documentation at https://mariadb.com/docs
    
    You can start the MariaDB daemon with:
    cd '/home/alan/partkeepr-ng/local-install/native/mariadb/usr' ; /home/alan/partkeepr-ng/local-install/native/mariadb/usr/bin/mariadbd-safe --datadir='/home/alan/partkeepr-ng/local-install/native/database'
    
    You can test the MariaDB daemon with mysql-test-run.pl
    cd '/home/alan/partkeepr-ng/local-install/native/mariadb/usr/share/mysql/mysql-test' ; perl mariadb-test-run.pl
    
    Please report any problems at https://mariadb.org/jira
    
    The latest information about MariaDB is available at https://mariadb.org/.
    
    Consider joining MariaDB's strong and vibrant community:
    https://mariadb.org/get-involved/
    
    ✓ • 1.09s

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import shutil
        root=Path('/home/alan/partkeepr-ng/local-install/native')
        (root/'bin').mkdir(exist_ok=True)
        shutil.copy2('local-install/bin/partkeepr-api',root/'bin/partkeepr-api')
        old=dict(line.split('=',1) for line in Path('local-install/app.env').read_text().splitlines())
        cred=dict(line.split('=',1) for line in Path('local-install/database.env').read_text().splitlines())
        old.update(DATABASE_URL='mysql://partkeepr_ng:'+cred['MARIADB_PASSWORD']+'@127.0.0.1:3307/partkeepr_ng',PARTKEEPR_STORAGE_DIR='/home/alan/partkeepr-ng/local-install/storage',PARTKEEPR_FRONTEND_DIR='/home/alan/partkeepr-ng/frontend',BIND_ADDR='0.0.0.0:8082')
        (root/'app.env').write_text(''.join(k+'='+v+'\n' for k,v in old.items()));(root/'app.env').chmod(0o600)
        (root/'mariadb.cnf').write_text(f'''[mariadbd]
        basedir={root}/mariadb/usr
        datadir={root}/database
        plugin-dir={root}/mariadb/usr/lib/mysql/plugin
        lc-messages-dir={root}/mariadb/usr/share/mysql
        socket=/run/user/1000/partkeepr-ng-db/mysql.sock
        pid-file=/run/user/1000/partkeepr-ng-db/mysql.pid
        bind-address=127.0.0.1
        port=3307
        skip-name-resolve
        character-set-server=utf8mb3
        collation-server=utf8mb3_unicode_ci
        innodb-buffer-pool-size=128M
        ''')
        (root/'partkeepr-ng-db.service').write_text(f'''[Unit]
        Description=PartKeepr NG native MariaDB
        
        [Service]
        Type=notify
        WorkingDirectory={root}
        Environment=LD_LIBRARY_PATH={root}/mariadb/usr/lib/x86_64-linux-gnu
        ExecStart={root}/mariadb/usr/sbin/mariadbd --defaults-file={root}/mariadb.cnf
        Restart=on-failure
        RestartSec=5
        TimeoutStartSec=120
        TimeoutStopSec=120
        RuntimeDirectory=partkeepr-ng-db
        RuntimeDirectoryMode=0700
        UMask=0077
        NoNewPrivileges=true
        PrivateTmp=true
        
        [Install]
        WantedBy=default.target
        ''')
        (root/'partkeepr-ng.service').write_text(f'''[Unit]
        Description=PartKeepr NG native web application
        Requires=partkeepr-ng-db.service
        After=partkeepr-ng-db.service
        
        [Service]
        Type=simple
        WorkingDirectory={root}
        EnvironmentFile={root}/app.env
        ExecStart={root}/bin/partkeepr-api
        Restart=on-failure
        RestartSec=5
        TimeoutStopSec=30
        UMask=0077
        NoNewPrivileges=true
        PrivateTmp=true
        
        [Install]
        WantedBy=default.target
        ''')
        print('Prepared native configuration and user service units.')
        PY
    Prepared native configuration and user service units.
    ✓ • 41ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import shutil
        units=Path('/home/alan/.config/systemd/user');units.mkdir(parents=True,exist_ok=True)
        for name in ['partkeepr-ng-db.service','partkeepr-ng.service']:
         dst=units/name
         if dst.exists(): raise SystemExit(f'Refusing to overwrite existing unit: {dst}')
         shutil.copy2(Path('local-install/native')/name,dst)
        PY
        systemctl --user daemon-reload
        systemctl --user start partkeepr-ng-db.service
        systemctl --user status partkeepr-ng-db.service --no-pager
    ● partkeepr-ng-db.service - PartKeepr NG native MariaDB
         Loaded: loaded (/home/alan/.config/systemd/user/partkeepr-ng-db.service; disabled; preset: enabled)
         Active: active (running) since Fri 2026-10-16 06:54:38 EDT; 8ms ago
       Main PID: 2082480 (mariadbd)
         Status: "Taking your SQL requests now..."
          Tasks: 12 (limit: 38303)
         Memory: 78.7M
            CPU: 77ms
         CGroup: /user.slice/user-1000.slice/user@1000.service/app.slice/partkeepr-ng-db.service
                 └─2082480 /home/alan/partkeepr-ng/local-install/native/mariadb/usr/sbin/mariadbd --defaults-file=/home/alan/partkeepr-ng/local-install/native/mariadb.cnf
    
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: File './ibtmp1' size is now 12.000MiB.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: log sequence number 44404; transaction id 14
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: Loading buffer pool(s) from /home/alan/partkeepr-ng/local-install/native/database/ib_buffer_pool
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] Plugin 'FEEDBACK' is disabled.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] Server socket created on IP: '127.0.0.1', port: '3307'.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: Buffer pool(s) load completed at 261016  6:54:38
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Warning] 'proxies_priv' entry '@% alan@debian' ignored in --skip-name-resolve mode.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] /home/alan/partkeepr-ng/local-install/native/mariadb/usr/sbin/mariadbd: ready for connections.
    Oct 16 06:54:38 debian mariadbd[2082480]: Version: '10.11.19-MariaDB-0+deb12u1'  socket: '/run/user/1000/partkeepr-ng-db/mysql.sock'  port: 3307  Debian 12
    Oct 16 06:54:38 debian systemd[1107]: Started partkeepr-ng-db.service - PartKeepr NG native MariaDB.
    ✓ • 569ms

## Assistant

The native MariaDB service is running successfully. Systemd’s user-service mode is available, and I’ve enabled it to start at boot and continue running after logout.

Next I’ll briefly stop the containerized app, back up its current database and files, and transfer the database to the native service. Then I’ll switch port 8082 to the native app and verify the data and login behavior.

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        root=Path('local-install/native')
        client="['/home/alan/partkeepr-ng/local-install/native/mariadb/usr/bin/mariadb','--no-defaults','--socket=/run/user/1000/partkeepr-ng-db/mysql.sock','-u','alan','-N','-B','partkeepr_ng']"
        for original,new in [('verify.py','verify.py'),('check-password-login.py','check-password-login.py')]:
         s=(Path('local-install')/original).read_text()
         lines=s.splitlines()
         lines=[('base='+client) if line.startswith('base=') else line for line in lines]
         s='\n'.join(lines)+'\n'
         s=s.replace('local-install/app.env','local-install/native/app.env').replace('local-install/verification.json','local-install/native/verification.json')
         (root/new).write_text(s)
        print('Prepared verification against native MariaDB.')
        PY
    Prepared verification against native MariaDB.
    ✓ • 14ms

## Activity

    $ cat > local-install/native/cutover.py <<'PY'
        from pathlib import Path
        import subprocess,datetime,json,os,tarfile
        root=Path('/home/alan/partkeepr-ng/local-install/native')
        compose=['docker','compose','-f','local-install/compose.yaml']
        def run(args,**kw): return subprocess.run(args,check=True,**kw)
        docker_sql=compose+['exec','-T','db','sh','-c','exec mariadb -N -B -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"']
        client=[str(root/'mariadb/usr/bin/mariadb'),'--no-defaults','--socket=/run/user/1000/partkeepr-ng-db/mysql.sock','-u','alan','-N','-B']
        def query(args,q):return run(args,input=q,text=True,capture_output=True).stdout
        stamp=datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
        backup=root/'backups'/stamp;backup.mkdir(mode=0o700)
        run(compose+['stop','app'])
        try:
         with (backup/'database.sql').open('wb') as f:
          run(compose+['exec','-T','db','sh','-c','exec mariadb-dump --single-transaction --routines --triggers -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"'],stdout=f)
         (backup/'database.sql').chmod(0o600)
         with tarfile.open(backup/'files.tar.gz','w:gz') as tar:tar.add('local-install/storage',arcname='storage')
         tables=query(docker_sql,'SHOW TABLES;').splitlines()
         checks=''.join('SELECT '+"'"+table+"',COUNT(*) FROM `"+table+'`;\n' for table in tables)
         before=query(docker_sql,checks)
         (backup/'table-counts.tsv').write_text(before)
         creds=dict(line.split('=',1) for line in Path('local-install/database.env').read_text().splitlines())
         # Credentials were generated as hexadecimal; no shell interpolation is used.
         query(client,"CREATE DATABASE partkeepr_ng CHARACTER SET utf8mb3 COLLATE utf8mb3_unicode_ci; CREATE USER 'partkeepr_ng'@'127.0.0.1' IDENTIFIED BY '"+creds['MARIADB_PASSWORD']+"'; GRANT ALL ON partkeepr_ng.* TO 'partkeepr_ng'@'127.0.0.1';")
         with (backup/'database.sql').open('rb') as dump:run(client+['partkeepr_ng'],stdin=dump)
         after=query(client+['partkeepr_ng'],checks)
         assert before==after,'Table counts differ after restore'
         checksum_query='CHECKSUM TABLE '+','.join('`'+table+'`' for table in tables)+' EXTENDED;'
         old_sum=query(docker_sql,checksum_query)
         new_sum=query(client+['partkeepr_ng'],checksum_query)
         (backup/'source-checksums.tsv').write_text(old_sum)
         (backup/'native-checksums.tsv').write_text(new_sum)
         assert old_sum==new_sum,'Table checksums differ after restore'
         (root/'cutover-report.json').write_text(json.dumps({'backup':str(backup),'table_count':len(tables),'all_table_counts_match':True,'all_table_checksums_match':True},indent=2)+'\n')
         run(['systemctl','--user','start','partkeepr-ng.service'])
         print('Native app started. All',len(tables),'table counts and checksums match. Backup:',backup)
        except BaseException:
         run(['systemctl','--user','stop','partkeepr-ng.service'])
         run(compose+['start','app'])
         raise
        PY
        python3 local-install/native/cutover.py
     Container partkeepr-ng-app-1  Stopping
     Container partkeepr-ng-app-1  Stopped
    Native app started. All 51 table counts and checksums match. Backup: /home/alan/partkeepr-ng/local-install/native/backups/20261016-065521
    ✓ • 21.64s

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        root=Path('/home/alan/partkeepr-ng/local-install/native')
        (root/'database').chmod(0o700)
        script='''#!/bin/bash
        set -euo pipefail
        umask 077
        root=/home/alan/partkeepr-ng/local-install/native
        backup="$root/backups/$(date +%Y%m%d-%H%M%S)"
        mkdir -p "$backup"
        was_running=0
        if systemctl --user is-active --quiet partkeepr-ng.service; then
            was_running=1
            systemctl --user stop partkeepr-ng.service
        fi
        finish() {
            if [[ "$was_running" == 1 ]]; then systemctl --user start partkeepr-ng.service; fi
        }
        trap finish EXIT
        "$root/mariadb/usr/bin/mariadb-dump" --no-defaults \\
            --socket=/run/user/1000/partkeepr-ng-db/mysql.sock -u alan \\
            --single-transaction --routines --triggers partkeepr_ng > "$backup/database.sql"
        tar -czf "$backup/files.tar.gz" -C /home/alan/partkeepr-ng/local-install storage
        cp "$root/app.env" "$root/mariadb.cnf" "$backup/"
        printf 'Backup saved: %s\\n' "$backup"
        '''
        (root/'backup.sh').write_text(script);(root/'backup.sh').chmod(0o700)
        PY
    ✓ • 12ms

## Activity

    $ python3 local-install/native/check-password-login.py; python3 local-install/native/verify.py; python3 - <<'PY'
        import urllib.request
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
        for host in ['127.0.0.1','10.0.0.1','192.168.5.109']:
         with opener.open('http://'+host+':8082/health',timeout=10) as r: print(host,r.status,r.read().decode())
        PY
        systemctl --user status partkeepr-ng.service partkeepr-ng-db.service --no-pager
    original single iteration: wrong password rejected, correct password accepted, session works
    5000 iterations: wrong password rejected, correct password accepted, session works
    Temporary test account removed; existing accounts unchanged.
    {'path': '/health', 'status': 200, 'bytes': 2}
    {'path': '/', 'status': 200, 'bytes': 1090}
    {'path': '/api/me (unauthenticated)', 'status': 401}
    {'path': '/api/me', 'status': 200, 'bytes': 48}
    {'path': '/api/version', 'status': 200, 'bytes': 20}
    {'path': '/api/parts', 'status': 200, 'bytes': 29574}
    {'path': '/api/part_categories/tree', 'status': 200, 'bytes': 27590}
    {'path': '/api/storage_tree', 'status': 200, 'bytes': 47090}
    {'path': '/api/footprint_tree', 'status': 200, 'bytes': 14752}
    {'path': '/api/projects', 'status': 200, 'bytes': 2}
    {'path': '/api/parts/1', 'status': 200, 'bytes': 1336}
    {'path': '/files/PartAttachment/1', 'status': 200, 'bytes': 324405}
    {'path': '/files/FootprintAttachment/1', 'status': 200, 'bytes': 138512}
    {'path': '/files/FootprintImage/1', 'status': 200, 'bytes': 23365}
    {'path': '/files/FootprintImage/1/thumb', 'status': 200, 'bytes': 10066}
    {'path': '/files/ManufacturerICLogo/1', 'status': 200, 'bytes': 2195}
    {'path': '/files/ManufacturerICLogo/1/thumb', 'status': 200, 'bytes': 3718}
    {'path': '/files/StorageLocationImage/1', 'status': 200, 'bytes': 3927785}
    {'path': '/files/StorageLocationImage/1/thumb', 'status': 200, 'bytes': 10860}
    127.0.0.1 200 ok
    10.0.0.1 200 ok
    192.168.5.109 200 ok
    ● partkeepr-ng.service - PartKeepr NG native web application
         Loaded: loaded (/home/alan/.config/systemd/user/partkeepr-ng.service; disabled; preset: enabled)
         Active: active (running) since Fri 2026-10-16 06:55:43 EDT; 14s ago
       Main PID: 2084034 (partkeepr-api)
          Tasks: 10 (limit: 38303)
         Memory: 3.6M
            CPU: 47ms
         CGroup: /user.slice/user-1000.slice/user@1000.service/app.slice/partkeepr-ng.service
                 └─2084034 /home/alan/partkeepr-ng/local-install/native/bin/partkeepr-api
    
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124140Z  INFO partkeepr_api::handlers::storage: file storage root root=/home/alan/partkeepr-ng/local-install/storage
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124163Z  WARN partkeepr_api::handlers::clamd: virus scanning disabled — set PARTKEEPR_CLAMD=tcp://host:port to enable
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124171Z  INFO partkeepr_api::handlers::printing: ptouch-print disabled (set PARTKEEPR_PTOUCH_BIN to enable)
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124178Z  INFO partkeepr_api::handlers::mouser: mouser search api disabled (set PARTKEEPR_MOUSER_SEARCH_KEY to enable)
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124190Z  INFO partkeepr_api::handlers::digikey: digikey api disabled (set PARTKEEPR_DIGIKEY_CLIENT_ID and PARTKEEPR_DIGIKEY_CLIENT_SECRET to enable)
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124198Z  INFO partkeepr_api::handlers::trustedparts: trustedparts api disabled (set PARTKEEPR_TRUSTEDPARTS_API_KEY to enable)
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124204Z  INFO partkeepr_api::handlers::jlcparts: jlcparts cross-reference disabled (set PARTKEEPR_JLCPARTS_DB_PATH to the mirrored cache.sqlite3 to enable)
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124211Z  INFO partkeepr_api: feature status — clamd=off ptouch=off mouser_search=off mouser_orders=off digikey=off trustedparts=off jlcparts=off
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.124224Z  INFO partkeepr_api: serving static frontend frontend_dir="/home/alan/partkeepr-ng/frontend"
    Oct 16 06:55:43 debian partkeepr-api[2084034]: 2026-10-16T10:55:43.125593Z  INFO partkeepr_api: listening bind_addr=0.0.0.0:8082
    
    ● partkeepr-ng-db.service - PartKeepr NG native MariaDB
         Loaded: loaded (/home/alan/.config/systemd/user/partkeepr-ng-db.service; disabled; preset: enabled)
         Active: active (running) since Fri 2026-10-16 06:54:38 EDT; 1min 18s ago
       Main PID: 2082480 (mariadbd)
         Status: "Taking your SQL requests now..."
          Tasks: 16 (limit: 38303)
         Memory: 95.2M
            CPU: 389ms
         CGroup: /user.slice/user-1000.slice/user@1000.service/app.slice/partkeepr-ng-db.service
                 └─2082480 /home/alan/partkeepr-ng/local-install/native/mariadb/usr/sbin/mariadbd --defaults-file=/home/alan/partkeepr-ng/local-install/native/mariadb.cnf
    
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: File './ibtmp1' size is now 12.000MiB.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: log sequence number 44404; transaction id 14
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: Loading buffer pool(s) from /home/alan/partkeepr-ng/local-install/native/database/ib_buffer_pool
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] Plugin 'FEEDBACK' is disabled.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] Server socket created on IP: '127.0.0.1', port: '3307'.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] InnoDB: Buffer pool(s) load completed at 261016  6:54:38
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Warning] 'proxies_priv' entry '@% alan@debian' ignored in --skip-name-resolve mode.
    Oct 16 06:54:38 debian mariadbd[2082480]: 2026-10-16  6:54:38 0 [Note] /home/alan/partkeepr-ng/local-install/native/mariadb/usr/sbin/mariadbd: ready for connections.
    Oct 16 06:54:38 debian mariadbd[2082480]: Version: '10.11.19-MariaDB-0+deb12u1'  socket: '/run/user/1000/partkeepr-ng-db/mysql.sock'  port: 3307  Debian 12
    Oct 16 06:54:38 debian systemd[1107]: Started partkeepr-ng-db.service - PartKeepr NG native MariaDB.
    ✓ • 511ms

## Activity

    $ systemctl --user enable partkeepr-ng-db.service partkeepr-ng.service; docker update --restart=no partkeepr-ng-app-1 partkeepr-ng-db-1; docker compose -f local-install/compose.yaml stop db; systemctl --user is-enabled partkeepr-ng.service partkeepr-ng-db.service; loginctl show-user alan -p Linger; docker ps --filter name=partkeepr-ng --format '{{.Names}} {{.Status}}'; systemctl is-active mysql
    Created symlink /home/alan/.config/systemd/user/default.target.wants/partkeepr-ng-db.service → /home/alan/.config/systemd/user/partkeepr-ng-db.service.
    Created symlink /home/alan/.config/systemd/user/default.target.wants/partkeepr-ng.service → /home/alan/.config/systemd/user/partkeepr-ng.service.
    partkeepr-ng-app-1
    partkeepr-ng-db-1
     Container partkeepr-ng-db-1  Stopping
     Container partkeepr-ng-db-1  Stopped
    enabled
    enabled
    Linger=yes
    active
    ✓ • 590ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('local-install/compose.yaml');p.write_text(p.read_text().replace('restart: unless-stopped','restart: "no"'))
        root=Path('/home/alan/partkeepr-ng/local-install/native')
        readme='''# Native PartKeepr NG installation
        
        The application and MariaDB now run natively under alan's systemd user manager.
        Both units are enabled, and loginctl linger is enabled, so they start at boot
        and continue running when alan logs out. Docker is not needed for this app.
        
        URLs:
        - WireGuard: http://10.0.0.1:8082/
        - LAN: http://192.168.5.109:8082/
        - On this server: http://localhost:8082/
        
        Use your existing PartKeepr username and password. The original password
        compatibility fix is included. The web server listens on IPv4 port 8082.
        MariaDB listens only on 127.0.0.1:3307. Existing native MySQL on 3306 is unchanged.
        
        ## Service management (as alan, no sudo)
        
            systemctl --user status partkeepr-ng partkeepr-ng-db
            systemctl --user restart partkeepr-ng
            journalctl --user -u partkeepr-ng -n 100
            journalctl --user -u partkeepr-ng-db -n 100
        
        To restart the database, stop the app first and start it afterward:
        
            systemctl --user stop partkeepr-ng
            systemctl --user restart partkeepr-ng-db
            systemctl --user start partkeepr-ng
        
        ## Paths
        
        - Service units: /home/alan/.config/systemd/user/partkeepr-ng{,-db}.service
        - App binary: local-install/native/bin/partkeepr-api
        - Private app configuration: local-install/native/app.env (mode 600)
        - MariaDB configuration: local-install/native/mariadb.cnf
        - MariaDB binaries and supporting files: local-install/native/mariadb/
        - Database data: local-install/native/database/ (mode 700)
        - Files and images: local-install/storage/
        - Static frontend: frontend/
        - Local backups: local-install/native/backups/
        
        All local-install paths are inside /home/alan/partkeepr-ng and gitignored.
        Do not delete or relocate this checkout without migrating the service paths
        and persistent data. Credentials are retained from the containerized setup.
        
        ## Why MariaDB is a private native installation
        
        The host already runs MySQL Community Server. Installing Debian's mariadb-server
        package through apt would remove the existing MySQL packages. Instead, official
        Debian MariaDB 10.11.19 packages were downloaded and extracted privately, without
        installing or removing system packages. This is a normal native mariadbd process,
        not a container. Its missing liburing dependency is also included privately.
        Debian package downloads were verified through apt's package metadata.
        
        These private MariaDB binaries are NOT automatically upgraded by apt upgrade.
        For security updates, download and extract matching current Debian 10.11 packages
        (mariadb-server-core, mariadb-server, mariadb-client-core, mariadb-common, liburing2)
        into a staging directory, back up, stop both services, replace the private binary
        tree, then start the database followed by the app. Verify shared-library
        requirements and release upgrade instructions before switching versions. Do not
        install the conflicting system mariadb-server package without planning for MySQL.
        The package archives used for this installation remain in native/packages/.
        
        ## Backups
        
        Run as alan:
        
            /home/alan/partkeepr-ng/local-install/native/backup.sh
        
        This briefly stops the app, dumps the native database, archives files, saves
        configuration, and restarts the app if it was running. It does not use Docker.
        Backups are local; copy the completed backup directory to your normal backup
        location for protection against loss of this machine.
        
        The pre-cutover snapshot and matching table checksums are recorded in
        native/cutover-report.json. All 51 table counts and checksums matched after import.
        Original source dump/archive are also unchanged.
        
        ## App updates
        
        From /home/alan/partkeepr-ng:
        
            cargo +stable build --locked --release --bin partkeepr-api
            systemctl --user stop partkeepr-ng
            cp target/release/partkeepr-api local-install/native/bin/partkeepr-api
            systemctl --user start partkeepr-ng
        
        Review migration requirements before updating. Do not rerun the six original
        migrations; they have already been applied.
        
        ## Previous Docker installation
        
        partkeepr-ng-app-1 and partkeepr-ng-db-1 are stopped and their automatic restart
        is disabled. The old database volume is retained as a rollback snapshot; it does
        not receive new writes. The old compose file is retained for reference; do not
        start it alongside the native services because the application port is the same.
        The original PartKeepr containers and other applications were not changed.
        '''
        Path('local-install/README.md').write_text(readme)
        print('Updated operating instructions for native services.')
        PY
    Updated operating instructions for native services.
    ✓ • 8ms

## Assistant

The native app is now serving on port 8082. All 51 database tables matched the container database by both row count and checksum. Login tests for both password formats passed, along with API, download, thumbnail, LAN, and VPN checks.

Both native services are enabled at boot. The replacement Docker containers are stopped, with automatic restart disabled; their database volume remains as a rollback copy. I’m checking the native backup procedure before finishing.

## Activity

    $ local-install/native/backup.sh; systemctl --user is-active partkeepr-ng partkeepr-ng-db; python3 - <<'PY'
        import urllib.request
        with urllib.request.urlopen('http://127.0.0.1:8082/health',timeout=10) as r: print('Health:',r.status,r.read().decode())
        PY
        ss -ltn '( sport = :3307 or sport = :8082 )'; git status --short
    local-install/native/backup.sh: line 16: /home/alan/partkeepr-ng/local-install/native/mariadb/usr/bin/mariadb-dump: No such file or directory
    active
    active
    Health: 200 ok
    State  Recv-Q Send-Q Local Address:Port Peer Address:PortProcess
    LISTEN 0      80         127.0.0.1:3307      0.0.0.0:*
    LISTEN 0      128          0.0.0.0:8082      0.0.0.0:*
     M README.md
     M backend/src/bin/reset-password.rs
     M backend/src/handlers/auth.rs
     M docs/migrating-from-partkeepr-1.4.0.md
    ✓ • 81ms

## Activity

    $ apt-get download mariadb-client
    Get:1 http://security.debian.org/debian-security bookworm-security/main amd64 mariadb-client amd64 1:10.11.19-0+deb12u1 [2952 kB]
    Fetched 2952 kB in 0s (8930 kB/s)
    ✓ • 377ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        import subprocess
        for p in Path('local-install/native/packages').glob('mariadb-client_*.deb'):
         subprocess.run(['dpkg-deb','-x',str(p),'local-install/native/mariadb'],check=True)
        p=Path('local-install/README.md');s=p.read_text().replace('mariadb-server-core, mariadb-server, mariadb-client-core, mariadb-common, liburing2','mariadb-server-core, mariadb-server, mariadb-client-core, mariadb-client, mariadb-common, liburing2');p.write_text(s)
        PY
        ldd local-install/native/mariadb/usr/bin/mariadb-dump
        linux-vdso.so.1 (0x00007ffca1f72000)
        libssl.so.3 => /lib/x86_64-linux-gnu/libssl.so.3 (0x00007f607b557000)
        libcrypto.so.3 => /lib/x86_64-linux-gnu/libcrypto.so.3 (0x00007f607b000000)
        libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x00007f607bafc000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f607ae1f000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f607bb3a000)
    ✓ • 210ms

## Activity

    $ local-install/native/backup.sh
    Backup saved: /home/alan/partkeepr-ng/local-install/native/backups/20261016-065717
    ✓ • 9.59s

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('local-install/native/backup.sh');s=p.read_text().replace('printf \'Backup saved: %s\\n\' "$backup"','touch "$backup/COMPLETE"\nprintf \'Backup saved: %s\\n\' "$backup"');p.write_text(s)
        for d in Path('local-install/native/backups').iterdir():
         sql=d/'database.sql'
         if sql.exists() and sql.stat().st_size==0:
          (d/'INCOMPLETE').write_text('Initial backup utility check failed before data was written. Do not use this directory as a backup.\n')
        print('Backup script records completion; incomplete initial check is labeled.')
        PY
    Backup script records completion; incomplete initial check is labeled.
    ✓ • 19ms

## Activity

    $ systemctl --user is-active partkeepr-ng.service partkeepr-ng-db.service; systemctl --user is-enabled partkeepr-ng.service partkeepr-ng-db.service; python3 - <<'PY'
        import urllib.request
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
        for host in ['10.0.0.1','192.168.5.109']:
         with opener.open('http://'+host+':8082/health',timeout=10) as r: print(host,r.status,r.read().decode())
        PY
        docker inspect --format '{{.Name}} running={{.State.Running}} restart={{.HostConfig.RestartPolicy.Name}}' partkeepr-ng-app-1 partkeepr-ng-db-1; systemctl is-active mysql
    active
    active
    enabled
    enabled
    10.0.0.1 200 ok
    192.168.5.109 200 ok
    /partkeepr-ng-app-1 running=false restart=no
    /partkeepr-ng-db-1 running=false restart=no
    active
    ✓ • 29ms

## Assistant

Done—**partkeepr-ng and MariaDB now run natively under systemd**, starting at boot and continuing after logout.

Use the same address: **[http://10.0.0.1:8082/](http://10.0.0.1:8082/)**.

- All 51 database tables matched after migration.
- Login, file downloads, thumbnails, and VPN/LAN access passed verification.
- The native backup procedure passed.
- The replacement Docker containers are stopped; their data remains for rollback.
- Your original PartKeepr and existing MySQL service are unchanged.

Manage the services as `alan`:

```bash
systemctl --user status partkeepr-ng partkeepr-ng-db
systemctl --user restart partkeepr-ng
journalctl --user -u partkeepr-ng -n 100
```

MariaDB uses private Debian binaries to avoid replacing the host’s existing MySQL. Those binaries require manual security updates.

Paths, backups, and maintenance instructions are in [local-install/README.md](/home/alan/partkeepr-ng/local-install/README.md).

## User

Can you add and commit changes doing so in a way separates as best as you can, the functional significance of changes you have made into different commits

## Assistant

I’ll review the changes and split them into commits by purpose. I’ll also add reusable systemd setup files so the native installation work is represented in Git without including your database, files, or credentials.

## Activity

    $ git status --short; git diff; rg --files scripts | head -50; cat local-install/native/partkeepr-ng.service local-install/native/partkeepr-ng-db.service local-install/native/backup.sh; cat .gitignore
     M README.md
     M backend/src/bin/reset-password.rs
     M backend/src/handlers/auth.rs
     M docs/migrating-from-partkeepr-1.4.0.md
    diff --git a/README.md b/README.md
    index 9a15289..09791b6 100644
    --- a/README.md
    +++ b/README.md
    @@ -38,7 +38,7 @@ The core PartKeepr 1.4.0 surface is fully preserved:
     - **Attachments** (datasheets, photos, BOM files) with optional clamd
       virus scanning, thumbnails, and inline upload-by-URL.
     - **Auth** compatible with the upstream FOSUserBundle password hashes
    -  (sha512 + 5000-iter + per-user salt), so existing user accounts log
    +  (SHA-512 + per-user salt; original single-iteration and 5000-iteration hashes), so existing user accounts log
       in unchanged.
    
     And several additions on top:
    diff --git a/backend/src/bin/reset-password.rs b/backend/src/bin/reset-password.rs
    index e636a08..d5382f9 100644
    --- a/backend/src/bin/reset-password.rs
    +++ b/backend/src/bin/reset-password.rs
    @@ -3,7 +3,7 @@
     //! `cargo run --bin reset-password -- <username> <new-password>`
     //!
     //! Resets a single FOSUser row's password hash to a new value, using
    -//! the same FOSUserBundle scheme the login handler verifies against
    +//! the 5000-iteration FOSUserBundle variant supported by the login handler
     //! (sha512 over `password{salt}` iterated 5000×, base64). Looks up
     //! by either `username` or `username_canonical` so casing doesn't
     //! matter.
    @@ -131,10 +131,14 @@ async fn main() -> ExitCode {
     /// digest = sha512(salted), then 4999 more iterations of
     /// sha512(prev_digest || salted), final result base64-encoded.
     ///
    -/// Mirrors `verify_fos_password` in `handlers/auth.rs`. Keep them in
    -/// sync; if either is changed, both must change.
    +/// The verifier in `handlers/auth.rs` accepts this variant as well as
    +/// original PartKeepr single-iteration hashes. Empty salts are omitted.
     fn hash_fos_password(password: &str, salt: &str) -> String {
    -    let salted = format!("{password}{{{salt}}}");
    +    let salted = if salt.is_empty() {
    +        password.to_owned()
    +    } else {
    +        format!("{password}{{{salt}}}")
    +    };
         let salted_bytes = salted.as_bytes();
         let mut digest = Sha512::digest(salted_bytes).to_vec();
         for _ in 1..5000 {
    diff --git a/backend/src/handlers/auth.rs b/backend/src/handlers/auth.rs
    index 2a324c4..b7216d8 100644
    --- a/backend/src/handlers/auth.rs
    +++ b/backend/src/handlers/auth.rs
    @@ -3,8 +3,8 @@
     //! Slice 10a: authentication.
     //!
     //! Login verifies against the legacy `FOSUser` table's password hash
    -//! (FOSUserBundle's default: sha512 over `password{salt}` iterated 5000
    -//! times, base64-encoded), then looks up the matching `PartKeeprUser`
    +//! (PartKeepr's sha512 over `password{salt}`, base64-encoded, or the
    +//! 5000-iteration variant used by earlier partkeepr-ng), then looks up the matching `PartKeeprUser`
     //! row by username — the legacy app-table that all the `user_id` FK
     //! columns point at. The session is an HMAC-signed payload stored in
     //! a `partkeepr_session` cookie; no DB session table.
    @@ -139,21 +139,29 @@ impl SessionSigner {
    
     /// Verify a candidate password against the stored FOSUser hash.
     ///
    -/// FOSUserBundle's default `MessageDigestPasswordEncoder`:
    +/// PartKeepr configures Symfony's `MessageDigestPasswordEncoder` with
    +/// one iteration. Also accept the 5000-iteration Symfony default used by
    +/// earlier partkeepr-ng password resets. Hashes do not encode the count.
     ///
     /// ```text
    -/// salted = password + "{" + salt + "}"
    +/// salted = password + "{" + salt + "}" (or password when salt is empty)
     /// digest = sha512(salted)
    -/// for i in 1..5000:           # 4999 more iterations, 5000 total
    +/// for i in 1..iterations:
     ///     digest = sha512(digest || salted)
     /// stored = base64(digest)
     /// ```
     ///
     /// Returns true iff `password` produces `stored_hash_b64`.
     pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
    -    let salted = format!("{password}{{{salt}}}");
    +    let salted = if salt.is_empty() {
    +        password.to_owned()
    +    } else {
    +        format!("{password}{{{salt}}}")
    +    };
         let salted_bytes = salted.as_bytes();
         let mut digest = Sha512::digest(salted_bytes).to_vec();
    +    let legacy = base64::engine::general_purpose::STANDARD.encode(&digest);
    +    let legacy_matches = legacy.as_bytes().ct_eq(stored_hash_b64.as_bytes());
         for _ in 1..5000 {
             let mut h = Sha512::new();
             h.update(&digest);
    @@ -161,7 +169,34 @@ pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) ->
             digest = h.finalize().to_vec();
         }
         let computed = base64::engine::general_purpose::STANDARD.encode(&digest);
    -    bool::from(computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
    +    bool::from(legacy_matches | computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
    +}
    +
    +#[cfg(test)]
    +mod password_tests {
    +    use super::verify_fos_password;
    +
    +    #[test]
    +    fn accepts_original_symfony_encoder_fixtures() {
    +        // Generated by the encoder in the original PartKeepr container,
    +        // using public fixture inputs, for both supported iteration counts.
    +        for (salt, hash) in [
    +            ("fixture-salt", "ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ=="),
    +            ("", "56Pet5rfF3IeVtwi7PY/i/RYiPiK3LqjvHoKZxDlneP760AzH47ELwwL7DA3dlezZ7nLYA8ET6scqvGGfms9sw=="),
    +            ("fixture-salt", "yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw=="),
    +            ("", "t6IMG9m1WloOJoNUlEHdxZncGEmOgYKjuZ2Cc132ouOL3jVIwlMBJ0h1Q48SzCSBR81PvCMFzI8Y0kwS/v0dDA=="),
    +        ] {
    +            assert!(verify_fos_password("migration-test-password", salt, hash));
    +            assert!(!verify_fos_password("incorrect-password", salt, hash));
    +        }
    +    }
    +
    +    #[test]
    +    fn rejects_invalid_hashes() {
    +        for hash in ["", "invalid", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="] {
    +            assert!(!verify_fos_password("migration-test-password", "fixture-salt", hash));
    +        }
    +    }
     }
    
     // ---------------------------------------------------------------------------
    diff --git a/docs/migrating-from-partkeepr-1.4.0.md b/docs/migrating-from-partkeepr-1.4.0.md
    index 01e8af2..ce71271 100644
    --- a/docs/migrating-from-partkeepr-1.4.0.md
    +++ b/docs/migrating-from-partkeepr-1.4.0.md
    @@ -20,7 +20,8 @@ goes wrong, you still have your original database to fall back to.
       schema.
     - All your `Part`, `PartCategory`, `StorageLocation`, `Manufacturer`,
       `Distributor`, `StockEntry`, `Project`, `ProjectRun`, etc., rows.
    -- `FOSUser` password hashes (sha512 + 5000 iter + per-user salt).
    +- `FOSUser` password hashes (SHA-512 + per-user salt; original
    +  single-iteration and 5000-iteration hashes are both accepted).
       Existing user accounts log in unchanged.
     - The on-disk file layout: `data/files/PartAttachment/{uuid}.{ext}`,
       `data/images/iclogo/{uuid}.{ext}`, etc. partkeepr-ng reads from the
    scripts/make-dist.sh
    scripts/analyze-issues.py
    scripts/dump-upstream-issues.sh
    scripts/import-legacy-files.sh
    scripts/migrations/0004-container-rows-backfill.sql
    scripts/migrations/0003-stock-entry-order-tracking.sql
    scripts/migrations/0001-attribution-backfill.sql
    scripts/migrations/0002-multi-location-parts.sql
    scripts/migrations/0006-stock-entry-container-tracking.sql
    scripts/migrations/0005-partunit-cleanup.sql
    scripts/extract-schema.py
    scripts/promote-to-bare.sh
    scripts/jlcparts-sync.sh
    scripts/generate-rust-models.py
    scripts/setup-jlcparts-paths.sh
    scripts/release.sh
    scripts/overlay-release.sh
    scripts/probe-mouser-orderhistory.sh
    scripts/migrate-from-upstream.sh
    scripts/jlcparts-build/LCSC-API.md
    scripts/jlcparts-build/README.md
    scripts/jlcparts-build/LICENSE.upstream
    scripts/jlcparts-build/setup.py
    scripts/jlcparts-build/jlcparts/lcsc.py
    scripts/jlcparts-build/jlcparts/common.py
    scripts/jlcparts-build/jlcparts/descriptionAttributes.py
    scripts/jlcparts-build/jlcparts/ui.py
    scripts/jlcparts-build/jlcparts/jlcpcb.py
    scripts/jlcparts-build/jlcparts/partLib.py
    scripts/jlcparts-build/jlcparts/datatables.py
    scripts/jlcparts-build/jlcparts/attributes.py
    scripts/jlcparts-build/jlcparts/webdb.py
    scripts/jlcparts-build/jlcparts/migrate.py
    scripts/jlcparts-build/jlcparts/__init__.py
    [Unit]
    Description=PartKeepr NG native web application
    Requires=partkeepr-ng-db.service
    After=partkeepr-ng-db.service
    
    [Service]
    Type=simple
    WorkingDirectory=/home/alan/partkeepr-ng/local-install/native
    EnvironmentFile=/home/alan/partkeepr-ng/local-install/native/app.env
    ExecStart=/home/alan/partkeepr-ng/local-install/native/bin/partkeepr-api
    Restart=on-failure
    RestartSec=5
    TimeoutStopSec=30
    UMask=0077
    NoNewPrivileges=true
    PrivateTmp=true
    
    [Install]
    WantedBy=default.target
    [Unit]
    Description=PartKeepr NG native MariaDB
    
    [Service]
    Type=notify
    WorkingDirectory=/home/alan/partkeepr-ng/local-install/native
    Environment=LD_LIBRARY_PATH=/home/alan/partkeepr-ng/local-install/native/mariadb/usr/lib/x86_64-linux-gnu
    ExecStart=/home/alan/partkeepr-ng/local-install/native/mariadb/usr/sbin/mariadbd --defaults-file=/home/alan/partkeepr-ng/local-install/native/mariadb.cnf
    Restart=on-failure
    RestartSec=5
    TimeoutStartSec=120
    TimeoutStopSec=120
    RuntimeDirectory=partkeepr-ng-db
    RuntimeDirectoryMode=0700
    UMask=0077
    NoNewPrivileges=true
    PrivateTmp=true
    
    [Install]
    WantedBy=default.target
    #!/bin/bash
    set -euo pipefail
    umask 077
    root=/home/alan/partkeepr-ng/local-install/native
    backup="$root/backups/$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$backup"
    was_running=0
    if systemctl --user is-active --quiet partkeepr-ng.service; then
        was_running=1
        systemctl --user stop partkeepr-ng.service
    fi
    finish() {
        if [[ "$was_running" == 1 ]]; then systemctl --user start partkeepr-ng.service; fi
    }
    trap finish EXIT
    "$root/mariadb/usr/bin/mariadb-dump" --no-defaults \
        --socket=/run/user/1000/partkeepr-ng-db/mysql.sock -u alan \
        --single-transaction --routines --triggers partkeepr_ng > "$backup/database.sql"
    tar -czf "$backup/files.tar.gz" -C /home/alan/partkeepr-ng/local-install storage
    cp "$root/app.env" "$root/mariadb.cnf" "$backup/"
    touch "$backup/COMPLETE"
    printf 'Backup saved: %s\n' "$backup"
    # Build outputs (Cargo puts the workspace target/ at the root; per-crate
    # target/ dirs may also exist if a crate was built standalone)
    target/
    backend/target/
    frontend/target/
    frontend/dist/
    
    # Dev secrets and local-only env
    backend/.env
    frontend/.env
    
    # Upstream reference clone — re-clone via:
    #   cd reference && git clone https://github.com/partkeepr/PartKeepr.git
    # (gitignored to avoid embedding a foreign git repo and to keep our
    # tree small. The 1272-issue archive in docs/upstream-issues/ is
    # tracked because it preserves data that may not be reachable on
    # GitHub forever.)
    reference/PartKeepr/
    
    # Live data dumps — user's actual inventory; not for source control.
    # The schema-only dump and migration list ARE tracked so anyone can
    # spin up a dev DB from a checkout.
    docs/live-schema/partkeepr-data.sql
    
    # Editor / OS junk
    **/*.swp
    **/*.swo
    .DS_Store
    *~
    ✓ • 0ms

## Activity

    $ cat scripts/import-legacy-files.sh; cat docs/migrating-from-partkeepr-1.4.0.md | sed -n '15,155p'; ls -l .git/hooks; git log -5 --oneline
    #!/bin/bash
    # Import attachment/image files from a legacy PartKeepr install into the
    # new partkeepr-ng storage tree.
    #
    # Legacy PartKeepr stores files under TWO trees within the install dir:
    #
    #     data/files/PartAttachment/       — datasheets, docs (per-Part)
    #     data/files/FootprintAttachment/  — per-Footprint non-image files
    #     data/images/footprint/           — per-Footprint image files
    #     data/images/iclogo/              — Manufacturer logos
    #     data/images/storagelocation/     — StorageLocation photos (uncommon)
    #
    # Filenames are `{uuid}.{ext}` UUIDs that match each table's `filename`
    # column. Our target layout flattens everything to one PascalCase
    # subdir per table:
    #
    #     $DEST/PartAttachment/{uuid}.{ext}
    #     $DEST/FootprintAttachment/{uuid}.{ext}
    #     $DEST/FootprintImage/{uuid}.{ext}
    #     $DEST/ManufacturerICLogo/{uuid}.{ext}
    #     $DEST/StorageLocationImage/{uuid}.{ext}
    #
    # Idempotent — safe to re-run before, during, and after cutover. Skips
    # missing source dirs cleanly.
    #
    # Usage:
    #     scripts/import-legacy-files.sh /local/path/to/data
    #     scripts/import-legacy-files.sh user@host:/some/path/data
    #
    # After rsync the script invokes clamdscan with --fdpass on the
    # destination, so files are virus-scanned before they can be served.
    #
    # The expected source layout is a PartKeepr 1.4.0 data directory:
    #   <SRC>/files/PartAttachment/  <SRC>/files/FootprintAttachment/  ...
    #   <SRC>/images/iclogo/         <SRC>/images/footprint/           ...
    
    set -euo pipefail
    
    if [ $# -lt 1 ]; then
        echo "usage: $0 <SOURCE>" >&2
        echo "  SOURCE: rsync-style path; e.g. /path/to/data or user@host:/path/to/data" >&2
        exit 2
    fi
    SRC="$1"
    DEST="${PARTKEEPR_STORAGE_DIR:-$HOME/.local/share/partkeepr-ng/files}"
    SRC_TRIMMED="${SRC%/}"
    
    echo "Source:      $SRC_TRIMMED"
    echo "Destination: $DEST"
    echo
    
    mkdir -p "$DEST"
    
    # Map of dest-table → relative source path inside the legacy data root.
    # Order in this list controls the order of progress output.
    TABLES=(
        "PartAttachment        files/PartAttachment"
        "FootprintAttachment   files/FootprintAttachment"
        "FootprintImage        images/footprint"
        "ManufacturerICLogo    images/iclogo"
        "StorageLocationImage  images/storagelocation"
    )
    
    for entry in "${TABLES[@]}"; do
        # Split each entry into table name + source subpath (any whitespace).
        read -r table src_sub <<< "$entry"
        src_path="$SRC_TRIMMED/$src_sub/"
        dest_path="$DEST/$table/"
        mkdir -p "$dest_path"
        echo "==> $table   <—  $src_sub"
        if ! rsync -ah --info=progress2 --ignore-missing-args \
                "$src_path" "$dest_path" 2>&1 | tail -3; then
            echo "    (skipped — source dir not present)"
        fi
    done
    
    echo
    echo "File counts now in $DEST:"
    for entry in "${TABLES[@]}"; do
        read -r table _ <<< "$entry"
        count=$(find "$DEST/$table" -maxdepth 1 -type f 2>/dev/null | wc -l)
        printf "  %-25s %d\n" "$table" "$count"
    done
    
    # --------------------------------------------------------------------------
    # Virus scan — legacy PartKeepr did NOT scan uploads.
    # --------------------------------------------------------------------------
    echo
    if command -v clamdscan >/dev/null 2>&1; then
        if ! clamdscan --version >/dev/null 2>&1; then
            echo "clamdscan installed but not responsive — skipping scan."
            echo "Run again once clamd is up, or scan manually:"
            echo "    clamdscan --fdpass --infected --multiscan \"$DEST\""
            exit 0
        fi
        echo "Scanning $DEST with clamdscan…"
        # --fdpass: clamdscan opens the file as the invoking user and
        #           hands the fd to clamd over the socket. Without this,
        #           clamd would try to open the file as the clamav user,
        #           which can't read files in $HOME/.local. fdpass is the
        #           portable answer for cross-user dev setups.
        if clamdscan --fdpass --infected --multiscan --no-summary "$DEST"; then
            echo "Scan complete — no infections found."
        else
            rc=$?
            echo
            echo "*** clamdscan reported issues (exit $rc). ***"
            echo "Exit codes: 1 = virus found, 2 = error / unreadable file."
            echo "Review the FOUND lines above and quarantine as needed."
            exit "$rc"
        fi
    else
        echo "clamdscan not installed — skipping scan."
        echo "When clamd is set up, re-run this script or scan manually:"
        echo "    clamdscan --fdpass --infected --multiscan \"$DEST\""
    fi
    goes wrong, you still have your original database to fall back to.
    
    ## What's preserved verbatim
    
    - Every table, column, index, and foreign key from the 49-table 1.4.0
      schema.
    - All your `Part`, `PartCategory`, `StorageLocation`, `Manufacturer`,
      `Distributor`, `StockEntry`, `Project`, `ProjectRun`, etc., rows.
    - `FOSUser` password hashes (SHA-512 + per-user salt; original
      single-iteration and 5000-iteration hashes are both accepted).
      Existing user accounts log in unchanged.
    - The on-disk file layout: `data/files/PartAttachment/{uuid}.{ext}`,
      `data/images/iclogo/{uuid}.{ext}`, etc. partkeepr-ng reads from the
      same paths as upstream.
    
    ## What's added
    
    Six additive migrations in `scripts/migrations/`, applied in numeric
    prefix order:
    
    | File | What it adds |
    |---|---|
    | `0001-attribution-backfill.sql` | Adds a "system" `PartKeeprUser` row and assigns it to legacy `Project`/`StockEntry` rows that had a NULL `user_id`. Lets every history row attribute to *someone*. |
    | `0002-multi-location-parts.sql` | New `PartStorageLocation` table — per-physical-container row breakdown of a part's stock (the "Reel A holds 1000, Loose pile holds 200" model). Adds nothing to existing tables. |
    | `0003-stock-entry-order-tracking.sql` | Adds nullable `StockEntry.distributor_id` + `salesOrderNumber` columns + index. Backfills receipts attributed to known sales orders. |
    | `0004-container-rows-backfill.sql` | Drops NOT NULL on `PartStorageLocation.storageLocation_id`, then backfills a `Loose` row covering existing stock for every part with stock but no PSL row. |
    | `0005-partunit-cleanup.sql` | Drops vestigial reel/cut-tape PartUnit rows (their role is now played by `PartStorageLocation.form`); seeds standard measurement units (Metres, Litres, Kilograms, etc.). |
    | `0006-stock-entry-container-tracking.sql` | Adds nullable `StockEntry.partStorageLocation_id` FK. Lets the system tie each receipt/draw to a specific physical container. |
    
    All migrations are additive (new tables, nullable columns, INSERTs).
    Running one twice fails loudly on the duplicate `ALTER TABLE` /
    `CREATE TABLE`, not silently — re-running the wrapper after a fix is
    generally safe, but expect noisy errors past the point of last success.
    
    ## Step-by-step
    
    ### 1. Dump prod
    
    On the legacy host:
    
    ```bash
    mysqldump -u <user> -p --single-transaction --routines --triggers \
        partkeepr > /tmp/partkeepr-prod.sql
    
    # The file storage tree (paths match your PartKeepr install):
    tar -czf /tmp/partkeepr-files.tar.gz -C /var/www/com.example.parts data/
    ```
    
    ### 2. Restore on the target host
    
    ```bash
    # Create a target DB and grant the new dev user.
    mysql -u root <<EOF
    CREATE DATABASE partkeepr_ng;
    CREATE USER 'partkeepr_ng'@'localhost' IDENTIFIED BY '<password>';
    GRANT ALL ON partkeepr_ng.* TO 'partkeepr_ng'@'localhost';
    EOF
    
    mysql -u partkeepr_ng -p partkeepr_ng < /tmp/partkeepr-prod.sql
    ```
    
    Sanity-check the restore:
    
    ```bash
    mysql -u partkeepr_ng -p partkeepr_ng -e "SELECT COUNT(*) FROM Part;"
    ```
    
    The number should match what you saw on the legacy host.
    
    ### 3. Apply additive migrations
    
    The wrapper script runs all migrations in order, with one-line progress
    output:
    
    ```bash
    ./scripts/migrate-from-upstream.sh \
        -h localhost -u partkeepr_ng -p '<password>' -d partkeepr_ng
    ```
    
    Or run them manually:
    
    ```bash
    for f in scripts/migrations/*.sql; do
        echo "--- $f ---"
        mysql -u partkeepr_ng -p partkeepr_ng < "$f"
    done
    ```
    
    (Glob order matches the dependency order — current file names sort
    correctly.)
    
    If a migration fails partway through, the rest of that file rolls back
    (`ALTER TABLE` is atomic per statement on InnoDB). Earlier migrations
    that succeeded stay applied. Re-running after fixing the issue is
    generally safe — failed `ALTER`s leave nothing behind.
    
    ### 4. Rsync the file storage tree
    
    ```bash
    # On the target:
    mkdir -p /var/lib/partkeepr-ng/files
    tar -xzf /tmp/partkeepr-files.tar.gz -C /var/lib/partkeepr-ng/
    
    # Files now live at /var/lib/partkeepr-ng/files/PartAttachment/...
    # images/ likewise.
    ```
    
    The `data/files/Temporary/` and `data/images/cache/` and
    `data/images/temp/` subtrees can be skipped — they're transient on the
    PartKeepr side and partkeepr-ng doesn't read them.
    
    ### 5. Configure and start the backend
    
    ```bash
    cp backend/.env.example backend/.env
    $EDITOR backend/.env
    ```
    
    Required:
    
    ```env
    DATABASE_URL=mysql://partkeepr_ng:<password>@localhost:3306/partkeepr_ng
    PARTKEEPR_SESSION_SECRET=<openssl rand -base64 48>
    PARTKEEPR_STORAGE_DIR=/var/lib/partkeepr-ng/files
    ```
    
    Optional (leave unset to disable the corresponding feature):
    
    ```env
    PARTKEEPR_CLAMD=tcp://127.0.0.1:3310           # virus scanning on uploads
    PARTKEEPR_PTOUCH_BIN=/usr/bin/ptouch-print     # Brother PT-D410 label printing
    PARTKEEPR_MOUSER_SEARCH_KEY=...                # Mouser API
    PARTKEEPR_MOUSER_ORDER_KEY=...
    PARTKEEPR_DIGIKEY_CLIENT_ID=...                # Digi-Key API
    PARTKEEPR_DIGIKEY_CLIENT_SECRET=...
    PARTKEEPR_DIGIKEY_CUSTOMER_ID=...              # required for OrderStatus only
    PARTKEEPR_TRUSTEDPARTS_API_KEY=...             # TrustedParts.com live compare
    ```
    
    Start the API:
    
    total 60
    -rwxr-xr-x 1 alan alan  478 Oct 16 06:24 applypatch-msg.sample
    -rwxr-xr-x 1 alan alan  896 Oct 16 06:24 commit-msg.sample
    -rwxr-xr-x 1 alan alan 4726 Oct 16 06:24 fsmonitor-watchman.sample
    -rwxr-xr-x 1 alan alan  189 Oct 16 06:24 post-update.sample
    -rwxr-xr-x 1 alan alan  424 Oct 16 06:24 pre-applypatch.sample
    -rwxr-xr-x 1 alan alan 1643 Oct 16 06:24 pre-commit.sample
    -rwxr-xr-x 1 alan alan  416 Oct 16 06:24 pre-merge-commit.sample
    -rwxr-xr-x 1 alan alan 1374 Oct 16 06:24 pre-push.sample
    -rwxr-xr-x 1 alan alan 4898 Oct 16 06:24 pre-rebase.sample
    -rwxr-xr-x 1 alan alan  544 Oct 16 06:24 pre-receive.sample
    -rwxr-xr-x 1 alan alan 1492 Oct 16 06:24 prepare-commit-msg.sample
    -rwxr-xr-x 1 alan alan 2783 Oct 16 06:24 push-to-checkout.sample
    -rwxr-xr-x 1 alan alan 3650 Oct 16 06:24 update.sample
    73ba2d5 build(release): add gitolite-first release tooling
    af03772 v0.1.13: show daemon version in the header
    b6617b3 verify overlay-and-mirror composite hook
    75a9857 retrigger mirror
    4859322 v0.1.12: generic vendor-lookup dialog titles
    ✓ • 0ms

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('README.md');s=p.read_text().replace('  (SHA-512 + per-user salt; original single-iteration and 5000-iteration hashes), so existing user accounts log\n  in unchanged.','  (SHA-512 + per-user salt; original single-iteration and 5000-iteration\n  hashes), so existing user accounts log in unchanged.');p.write_text(s)
        p=Path('backend/src/handlers/auth.rs');s=p.read_text().replace('//! 5000-iteration variant used by earlier partkeepr-ng), then looks up the matching `PartKeeprUser`\n//! row by username','//! 5000-iteration variant used by earlier partkeepr-ng), then looks up\n//! the matching `PartKeeprUser` row by username');p.write_text(s)
        p=Path('backend/src/bin/reset-password.rs');s=p.read_text()+'''
        #[cfg(test)]
        mod tests {
            use super::hash_fos_password;
        
            #[test]
            fn matches_symfony_5000_iteration_fixtures() {
                assert_eq!(
                    hash_fos_password("migration-test-password", "fixture-salt"),
                    "yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw=="
                );
                assert_eq!(
                    hash_fos_password("migration-test-password", ""),
                    "t6IMG9m1WloOJoNUlEHdxZncGEmOgYKjuZ2Cc132ouOL3jVIwlMBJ0h1Q48SzCSBR81PvCMFzI8Y0kwS/v0dDA=="
                );
            }
        }
        ''';p.write_text(s)
        PY
        cargo +stable test --locked --release --bin partkeepr-api password_tests
        cargo +stable test --locked --release --bin reset-password
       Compiling partkeepr-api v0.1.13 (/home/alan/partkeepr-ng/backend)
        Finished `release` profile [optimized] target(s) in 8.53s
         Running unittests src/main.rs (target/release/deps/partkeepr_api-bf4fc9b8b712b94c)
    
    running 2 tests
    test handlers::auth::password_tests::rejects_invalid_hashes ... ok
    test handlers::auth::password_tests::accepts_original_symfony_encoder_fixtures ... ok
    
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
    
       Compiling partkeepr-api v0.1.13 (/home/alan/partkeepr-ng/backend)
        Finished `release` profile [optimized] target(s) in 0.38s
         Running unittests src/bin/reset-password.rs (target/release/deps/reset_password-3cb337cb545d2d24)
    
    running 1 test
    test tests::matches_symfony_5000_iteration_fixtures ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    ✓ • 9.02s

## Activity

    $ git add README.md backend/src/handlers/auth.rs backend/src/bin/reset-password.rs docs/migrating-from-partkeepr-1.4.0.md && git commit -m "fix(auth): accept original PartKeepr password hashes"
    [master 7abf1f5] fix(auth): accept original PartKeepr password hashes
     4 files changed, 72 insertions(+), 15 deletions(-)
    ✓ • 0ms

## Assistant

The first commit contains the password compatibility fix, regression tests, and related documentation. Next I’ll separate the migration-guide corrections from the native systemd deployment files.

## Activity

    $ python3 - <<'PY'
        from pathlib import Path
        p=Path('docs/migrating-from-partkeepr-1.4.0.md');s=p.read_text()
        s=s.replace('No data is touched destructively. The migration adds new tables and\nnullable columns; existing rows keep working unchanged. If something\ngoes wrong, you still have your original database to fall back to.','Always migrate a copy of the database. Most migrations add tables or\nnullable columns, but some update existing rows, and migration `0005`\nreassigns reel/cut-tape units and deletes specific unit rows. Your\noriginal database remains available for rollback when you work on a copy.')
        s=s.replace('- The on-disk file layout: `data/files/PartAttachment/{uuid}.{ext}`,\n  `data/images/iclogo/{uuid}.{ext}`, etc. partkeepr-ng reads from the\n  same paths as upstream.','- Original attachment and image filenames. The import maps upstream\n  directories into the entity-named directories expected by partkeepr-ng;\n  see step 4. Files do not need to be renamed.')
        s=s.replace('All migrations are additive (new tables, nullable columns, INSERTs).\nRunning one twice fails loudly on the duplicate `ALTER TABLE` /\n`CREATE TABLE`, not silently — re-running the wrapper after a fix is\ngenerally safe, but expect noisy errors past the point of last success.','Apply the migrations in order, once, to the restored copy. Several are\nidempotent, but `0006` is not: repeating it can fail on a duplicate column,\nindex, or foreign key. The wrapper does not track which files were applied.')
        s=s.replace('If a migration fails partway through, the rest of that file rolls back\n(`ALTER TABLE` is atomic per statement on InnoDB). Earlier migrations\nthat succeeded stay applied. Re-running after fixing the issue is\ngenerally safe — failed `ALTER`s leave nothing behind.','If a migration fails partway through, earlier statements can remain\napplied: a SQL file is not one transaction, and DDL statements can commit\nimplicitly. Inspect the database before resuming. Apply only the remaining\nstatements, or restore a fresh copy and start again.')
        a=s.index('### 4. Rsync the file storage tree');b=s.index('### 5. Configure and start the backend',a)
        s=s[:a]+'''### 4. Import attachments and images
        
        An archive created with `tar -czf ... -C /app data` contains a top-level
        `data/` directory. Extract it into a staging directory first:
        
        ```bash
        mkdir -p /tmp/partkeepr-import
        tar --no-same-owner -xzf /tmp/partkeepr-files.tar.gz -C /tmp/partkeepr-import
        
        export PARTKEEPR_STORAGE_DIR="$HOME/.local/share/partkeepr-ng/files"
        bash scripts/import-legacy-files.sh /tmp/partkeepr-import/data
        ```
        
        The existing import script maps these paths, preserving filenames:
        
        | Original directory under `data/` | Directory under `PARTKEEPR_STORAGE_DIR` |
        |---|---|
        | `files/PartAttachment` | `PartAttachment` |
        | `files/FootprintAttachment` | `FootprintAttachment` |
        | `images/footprint` | `FootprintImage` |
        | `images/iclogo` | `ManufacturerICLogo` |
        | `images/storagelocation` | `StorageLocationImage` |
        
        If your archive also includes `files/ProjectAttachment`, copy it separately:
        
        ```bash
        if [ -d /tmp/partkeepr-import/data/files/ProjectAttachment ]; then
            mkdir -p "$PARTKEEPR_STORAGE_DIR/ProjectAttachment"
            rsync -a /tmp/partkeepr-import/data/files/ProjectAttachment/ \\
                "$PARTKEEPR_STORAGE_DIR/ProjectAttachment/"
        fi
        ```
        
        The backend reads, for example,
        `$PARTKEEPR_STORAGE_DIR/ManufacturerICLogo/{uuid}.{ext}`; pointing it at
        an unmodified `data/` or `data/files/` tree will not find legacy images.
        Set the same storage directory in your backend configuration and ensure
        the account running the backend can read and write it. The backend
        regenerates thumbnails when requested. Temporary files and image caches
        are not required; retain the original archive as a backup.
        
        '''+s[b:]
        s=s.replace('PARTKEEPR_STORAGE_DIR=/var/lib/partkeepr-ng/files','PARTKEEPR_STORAGE_DIR=/absolute/path/to/your/imported/files')
        s=s.replace('partkeepr-ng never deletes data and never modifies upstream columns. If\nyou decide to go back to PartKeepr 1.4.0:', 'The safest rollback is the untouched original database and file tree.\nDropping the added tables and columns does not undo migration backfills,\nunit cleanup, or edits made through the new app. If you decide to go back\nto PartKeepr 1.4.0:')
        p.write_text(s)
        PY
        git diff --check; git diff --stat
     docs/migrating-from-partkeepr-1.4.0.md | 80 +++++++++++++++++++++++-----------
     1 file changed, 54 insertions(+), 26 deletions(-)
    ✓ • 105ms

## Activity

    $ git add docs/migrating-from-partkeepr-1.4.0.md && git commit -m "docs(migration): correct legacy file layout and recovery guidance"
    [master a1207ed] docs(migration): correct legacy file layout and recovery guidance
     1 file changed, 54 insertions(+), 26 deletions(-)
    ✓ • 0ms

## Activity

    $ mkdir -p contrib/systemd
        python3 - <<'PY'
        from pathlib import Path
        src=Path('local-install/native');dst=Path('contrib/systemd')
        for name in ['partkeepr-ng.service','partkeepr-ng-db.service']:
         (dst/name).write_text((src/name).read_text().replace('/home/alan','%h'))
        s=(src/'mariadb.cnf').read_text().replace('/home/alan/partkeepr-ng','@CHECKOUT@').replace('/run/user/1000','@RUNTIME_DIR@')
        (dst/'mariadb.cnf.example').write_text(s)
        (dst/'app.env.example').write_text('''# Render @CHECKOUT@ as the absolute path to your checkout.
        # Choose a long alphanumeric DB password, or URL-encode special characters.
        DATABASE_URL=mysql://partkeepr_ng:REPLACE_WITH_DATABASE_PASSWORD@127.0.0.1:3307/partkeepr_ng
        # Generate with: openssl rand -hex 48
        PARTKEEPR_SESSION_SECRET=REPLACE_WITH_GENERATED_SECRET
        PARTKEEPR_STORAGE_DIR=@CHECKOUT@/local-install/storage
        PARTKEEPR_FRONTEND_DIR=@CHECKOUT@/frontend
        # Use 127.0.0.1:8082 for access only from this machine.
        BIND_ADDR=0.0.0.0:8082
        RUST_LOG=info
        ''')
        p=Path('.gitignore');s=p.read_text().replace('frontend/.env\n','frontend/.env\n# Native deployment credentials, databases, uploads, packages, and backups.\n/local-install/\n');p.write_text(s)
        PY
        cat > scripts/backup-native.sh <<'SH'
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
        SH
        chmod +x scripts/backup-native.sh
        cat > docs/native-systemd.md <<'MD'
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
        MD
        python3 - <<'PY'
        from pathlib import Path
        p=Path('README.md');s=p.read_text().replace('## Optional features — provisioning','## Native deployment\n\nDocker is optional. The app and MariaDB can run as persistent systemd user\nservices; see [native deployment](docs/native-systemd.md) for service templates,\nconfiguration, backups, and an isolated MariaDB setup alongside existing MySQL.\n\n## Optional features — provisioning');p.write_text(s)
        PY
        bash -n scripts/backup-native.sh
        git diff --check
        git status --short
     M .gitignore
     M README.md
    ?? contrib/
    ?? docs/native-systemd.md
    ?? scripts/backup-native.sh
    ✓ • 115ms

## Activity

    $ systemd-analyze --user verify contrib/systemd/partkeepr-ng.service contrib/systemd/partkeepr-ng-db.service
    ✓ • 0ms

## Assistant

The changes now fall into three groups: password compatibility, migration-guide corrections, and native systemd deployment with backups. The deployment files use templates, and the private `local-install/` directory is explicitly ignored. Password tests, shell syntax checks, and systemd unit validation have passed.

## Activity

    $ git add .gitignore README.md contrib/systemd/partkeepr-ng.service contrib/systemd/partkeepr-ng-db.service contrib/systemd/mariadb.cnf.example contrib/systemd/app.env.example docs/native-systemd.md scripts/backup-native.sh && git diff --cached --check && git diff --cached --stat && git diff --cached --name-only
     .gitignore                              |   2 +
     README.md                               |   6 ++
     contrib/systemd/app.env.example         |  10 ++
     contrib/systemd/mariadb.cnf.example     |  13 +++
     contrib/systemd/partkeepr-ng-db.service |  20 ++++
     contrib/systemd/partkeepr-ng.service    |  19 ++++
     docs/native-systemd.md                  | 156 ++++++++++++++++++++++++++++++++
     scripts/backup-native.sh                |  41 +++++++++
     8 files changed, 267 insertions(+)
    .gitignore
    README.md
    contrib/systemd/app.env.example
    contrib/systemd/mariadb.cnf.example
    contrib/systemd/partkeepr-ng-db.service
    contrib/systemd/partkeepr-ng.service
    docs/native-systemd.md
    scripts/backup-native.sh
    ✓ • 0ms

## Activity

    $ git commit -m "feat(deploy): add native systemd services and backup workflow" && git status --short && git log -3 --oneline
    [master 8b6f1de] feat(deploy): add native systemd services and backup workflow
     8 files changed, 267 insertions(+)
     create mode 100644 contrib/systemd/app.env.example
     create mode 100644 contrib/systemd/mariadb.cnf.example
     create mode 100644 contrib/systemd/partkeepr-ng-db.service
     create mode 100644 contrib/systemd/partkeepr-ng.service
     create mode 100644 docs/native-systemd.md
     create mode 100755 scripts/backup-native.sh
    8b6f1de feat(deploy): add native systemd services and backup workflow
    a1207ed docs(migration): correct legacy file layout and recovery guidance
    7abf1f5 fix(auth): accept original PartKeepr password hashes
    ✓ • 0ms

## Assistant

Created three focused commits:

- `7abf1f5` — Password compatibility fix, regression tests, and auth documentation.
- `a1207ed` — Migration file-layout and recovery guidance corrections.
- `8b6f1de` — Native systemd templates, backup script, and deployment documentation.

Tests and validation passed. Working tree is clean. Credentials, databases, images, and backups remain excluded from Git. Nothing was pushed.
