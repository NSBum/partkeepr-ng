// SPDX-License-Identifier: GPL-3.0-or-later

//! User-owned, hashed, revocable bearer credentials. Management is session-only.
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use super::auth::CurrentUser;
use crate::error::AppError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KeyInfo {
    pub id: i64,
    pub user_id: i32,
    pub username: String,
    pub name: String,
    pub key_prefix: String,
    pub access: String,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub last_used_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub expired: bool,
}
const KEY_FIELDS: &str = "k.id, k.user_id, p.username, k.name, CONVERT(k.key_prefix USING utf8mb4) AS key_prefix, k.access, \
    k.created_at, k.expires_at, k.last_used_at, k.revoked_at, \
    (k.expires_at IS NOT NULL AND k.expires_at <= UTC_TIMESTAMP()) AS expired";

pub async fn list(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<KeyInfo>>, AppError> {
    let rows = sqlx::query_as::<_, KeyInfo>(&format!(
        "SELECT {KEY_FIELDS} FROM ApiKey k JOIN PartKeeprUser p ON p.id = k.user_id \
         WHERE k.user_id = ? OR ? ORDER BY k.id DESC"
    ))
    .bind(user.user_id)
    .bind(user.is_admin)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Owner {
    user_id: i32,
    username: String,
}
pub async fn owners(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<Owner>>, AppError> {
    let rows = sqlx::query_as::<_, Owner>(
        "SELECT p.id AS user_id, p.username FROM PartKeeprUser p \
         JOIN FOSUser f ON f.username = p.username \
         WHERE p.active = 1 AND f.enabled = 1 AND f.locked = 0 AND (p.id = ? OR ?) ORDER BY p.username")
        .bind(user.user_id).bind(user.is_admin).fetch_all(&pool).await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateKey {
    name: String,
    access: String,
    expires_in_days: Option<u32>,
    user_id: Option<i32>,
}
fn validate_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err(AppError::BadRequest("Use a key name of 1–100 characters."));
    }
    Ok(name)
}
impl CreateKey {
    fn validate(&self) -> Result<(), AppError> {
        validate_name(&self.name)?;
        if !["read", "read_write"].contains(&self.access.as_str()) {
            return Err(AppError::BadRequest(
                "Choose read-only or read/write access.",
            ));
        }
        if self
            .expires_in_days
            .is_some_and(|days| !(1..=3650).contains(&days))
        {
            return Err(AppError::BadRequest(
                "Expiry must be 1–3650 days, or null for no expiry.",
            ));
        }
        Ok(())
    }
}

fn generate_key() -> String {
    // UUID v4 uses the OS random source; two UUIDs provide 244 random bits.
    format!(
        "pkng_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn key_hash(key: &str) -> String {
    format!("{:x}", Sha256::digest(key.as_bytes()))
}
fn valid_key_format(key: &str) -> bool {
    key.len() == 69
        && key.starts_with("pkng_")
        && key.as_bytes()[5..].iter().all(u8::is_ascii_hexdigit)
}

#[derive(Serialize)]
struct CreatedKey {
    key: KeyInfo,
    api_key: String,
}
pub async fn create(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateKey>,
) -> Result<Response, AppError> {
    req.validate()?;
    let owner = req.user_id.unwrap_or(user.user_id);
    if owner != user.user_id && !user.is_admin {
        return Err(AppError::Forbidden(
            "Only administrators can assign keys to another account.",
        ));
    }
    let mut tx = pool.begin().await?;
    // Lock the owner so concurrent requests cannot bypass the active-key limit.
    let fos_id: Option<i32> = sqlx::query_scalar(
        "SELECT f.id FROM PartKeeprUser p JOIN FOSUser f ON f.username = p.username \
         WHERE p.id = ? AND p.active = 1 AND f.enabled = 1 AND f.locked = 0 FOR UPDATE",
    )
    .bind(owner)
    .fetch_optional(&mut *tx)
    .await?;
    let fos_id = fos_id.ok_or(AppError::BadRequest("Choose an active account."))?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ApiKey WHERE user_id = ? AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > UTC_TIMESTAMP())")
        .bind(owner).fetch_one(&mut *tx).await?;
    if count >= 100 {
        return Err(AppError::BadRequest(
            "Revoke an existing key before creating more (100 active keys per account).",
        ));
    }
    let api_key = generate_key();
    let expires_at = req
        .expires_in_days
        .map(|days| (Utc::now() + Duration::days(days as i64)).naive_utc());
    let id = sqlx::query("INSERT INTO ApiKey (user_id, fos_user_id, created_by, name, key_prefix, key_hash, access, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?, UTC_TIMESTAMP(), ?)")
        .bind(owner).bind(fos_id).bind(user.user_id).bind(req.name.trim()).bind(&api_key[..17])
        .bind(key_hash(&api_key)).bind(&req.access).bind(expires_at).execute(&mut *tx).await?.last_insert_id() as i64;
    let key = sqlx::query_as::<_, KeyInfo>(&format!(
        "SELECT {KEY_FIELDS} FROM ApiKey k JOIN PartKeeprUser p ON p.id = k.user_id WHERE k.id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(CreatedKey { key, api_key }),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameKey {
    name: String,
}
pub async fn rename(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<RenameKey>,
) -> Result<StatusCode, AppError> {
    let name = validate_name(&req.name)?;
    // SELECT first: MariaDB reports zero changed rows for an unchanged name.
    let allowed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ApiKey WHERE id = ? AND (user_id = ? OR ?)")
            .bind(id)
            .bind(user.user_id)
            .bind(user.is_admin)
            .fetch_one(&pool)
            .await?;
    if allowed == 0 {
        return Err(AppError::NotFound("API key"));
    }
    sqlx::query("UPDATE ApiKey SET name = ? WHERE id = ? AND (user_id = ? OR ?)")
        .bind(name)
        .bind(id)
        .bind(user.user_id)
        .bind(user.is_admin)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn revoke(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("UPDATE ApiKey SET revoked_at = COALESCE(revoked_at, UTC_TIMESTAMP()) WHERE id = ? AND (user_id = ? OR ?)")
        .bind(id).bind(user.user_id).bind(user.is_admin).execute(&pool).await?;
    // Check existence too, so revocation is idempotent with changed-row counts.
    if result.rows_affected() == 0 {
        let allowed: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ApiKey WHERE id = ? AND (user_id = ? OR ?)")
                .bind(id)
                .bind(user.user_id)
                .bind(user.is_admin)
                .fetch_one(&pool)
                .await?;
        if allowed == 0 {
            return Err(AppError::NotFound("API key"));
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(sqlx::FromRow)]
struct Principal {
    id: i64,
    user_id: i32,
    fos_user_id: i32,
    username: String,
    is_admin: bool,
    access: String,
}
pub async fn authenticate(
    pool: &MySqlPool,
    key: &str,
) -> Result<(CurrentUser, bool, i64), AppError> {
    if !valid_key_format(key) {
        return Err(AppError::Unauthorized("Invalid or inactive API key."));
    }
    let principal = sqlx::query_as::<_, Principal>(
        "SELECT k.id, k.user_id, k.fos_user_id, p.username, p.admin AS is_admin, k.access FROM ApiKey k \
         JOIN PartKeeprUser p ON p.id = k.user_id JOIN FOSUser f ON f.id = k.fos_user_id AND f.username = p.username \
         WHERE k.key_hash = ? AND k.revoked_at IS NULL AND (k.expires_at IS NULL OR k.expires_at > UTC_TIMESTAMP()) \
         AND p.active = 1 AND f.enabled = 1 AND f.locked = 0")
        .bind(key_hash(key)).fetch_optional(pool).await?
        .ok_or(AppError::Unauthorized("Invalid or inactive API key."))?;
    Ok((
        CurrentUser {
            user_id: principal.user_id,
            fos_user_id: principal.fos_user_id,
            username: principal.username,
            is_admin: principal.is_admin,
            exp: i64::MAX,
            password_version: String::new(),
        },
        principal.access == "read",
        principal.id,
    ))
}

pub async fn record_use(pool: &MySqlPool, id: i64) -> Result<(), AppError> {
    sqlx::query("UPDATE ApiKey SET last_used_at = UTC_TIMESTAMP() WHERE id = ? AND (last_used_at IS NULL OR last_used_at < UTC_TIMESTAMP() - INTERVAL 1 MINUTE)")
        .bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_are_distinct_and_validated_before_lookup() {
        let first = generate_key();
        let second = generate_key();
        assert_ne!(first, second);
        assert!(valid_key_format(&first));
        assert_eq!(key_hash(&first).len(), 64);
        assert_ne!(key_hash(&first), key_hash(&second));
        for key in [
            "",
            "pkng_",
            "not-a-key",
            &format!("{}x", first),
            &format!("pkng_{}", "z".repeat(64)),
        ] {
            assert!(!valid_key_format(key));
        }
    }
    #[test]
    fn validates_metadata() {
        let mut req = CreateKey {
            name: "  Stock script  ".into(),
            access: "read".into(),
            expires_in_days: Some(90),
            user_id: None,
        };
        assert!(req.validate().is_ok());
        assert_eq!(validate_name(&req.name).unwrap(), "Stock script");
        req.expires_in_days = Some(0);
        assert!(req.validate().is_err());
        req.expires_in_days = None;
        req.access = "admin".into();
        assert!(req.validate().is_err());
        assert!(validate_name("\n").is_err());
        assert!(validate_name(&"a".repeat(101)).is_err());
        assert!(serde_json::from_str::<RenameKey>(r#"{"name":"key","user_id":2}"#).is_err());
    }
}
