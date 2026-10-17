-- Revocable API credentials, separate from browser sessions and legacy tables.
-- Only a SHA-256 hash and identifying prefix are stored, never the full key.
CREATE TABLE IF NOT EXISTS ApiKey (
    id BIGINT NOT NULL AUTO_INCREMENT,
    user_id INT(11) NOT NULL,
    fos_user_id INT(11) NOT NULL,
    created_by INT(11) DEFAULT NULL,
    name VARCHAR(100) NOT NULL,
    key_prefix VARCHAR(17) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
    key_hash CHAR(64) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
    access ENUM('read', 'read_write') NOT NULL DEFAULT 'read',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME DEFAULT NULL,
    last_used_at DATETIME DEFAULT NULL,
    revoked_at DATETIME DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uniq_api_key_hash (key_hash),
    KEY idx_api_key_user (user_id),
    CONSTRAINT fk_api_key_user FOREIGN KEY (user_id) REFERENCES PartKeeprUser(id) ON DELETE CASCADE,
    CONSTRAINT fk_api_key_fos_user FOREIGN KEY (fos_user_id) REFERENCES FOSUser(id) ON DELETE CASCADE,
    CONSTRAINT fk_api_key_creator FOREIGN KEY (created_by) REFERENCES PartKeeprUser(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
