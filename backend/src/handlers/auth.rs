// SPDX-License-Identifier: GPL-3.0-or-later

//! Slice 10a: authentication.
//!
//! Login verifies against the legacy `FOSUser` table's password hash
//! (PartKeepr's sha512 over `password{salt}`, base64-encoded, or the
//! 5000-iteration variant used by earlier partkeepr-ng), then looks up
//! the matching `PartKeeprUser` row by username — the legacy app-table that all the `user_id` FK
//! columns point at. The session is an HMAC-signed payload stored in
//! a `partkeepr_session` cookie; no DB session table.
//!
//! The middleware in `auth_middleware` runs ahead of every other
//! handler. Endpoints in `PUBLIC_PATHS` (`/health`, `/api/login`)
//! pass through unauthenticated; everything else returns 401 unless
//! the cookie carries a valid, unexpired session.

use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
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

/// Separate provenance from identity so a bearer credential cannot manage accounts.
#[derive(Clone, Copy, PartialEq)]
pub enum AuthMethod {
    Session,
    ApiKey,
}

pub async fn session_only(req: Request, next: Next) -> Response {
    if req.extensions().get::<AuthMethod>() != Some(&AuthMethod::Session) {
        return AppError::Forbidden(
            "Sign in with your password to manage account settings and API keys.",
        )
        .into_response();
    }
    next.run(req).await
}

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
    /// Hash of the stored password hash; password changes revoke old cookies.
    #[serde(default)]
    pub password_version: String,
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
        Ok(Self {
            secret: s.into_bytes(),
        })
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
        let expected_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(expected_sig);
        if !bool::from(expected_b64.as_bytes().ct_eq(sig_b64.as_bytes())) {
            return None;
        }
        let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .ok()?;
        let payload: CurrentUser = serde_json::from_slice(&json_bytes).ok()?;
        if payload.is_expired() {
            return None;
        }
        Some(payload)
    }
}

// ---------------------------------------------------------------------------
//  FOSUserBundle password verification
// ---------------------------------------------------------------------------

/// Verify a candidate password against the stored FOSUser hash.
///
/// PartKeepr configures Symfony's `MessageDigestPasswordEncoder` with
/// one iteration. Also accept the 5000-iteration Symfony default used by
/// earlier partkeepr-ng password resets. Hashes do not encode the count.
///
/// ```text
/// salted = password + "{" + salt + "}" (or password when salt is empty)
/// digest = sha512(salted)
/// for i in 1..iterations:
///     digest = sha512(digest || salted)
/// stored = base64(digest)
/// ```
///
/// Returns true iff `password` produces `stored_hash_b64`.
pub fn verify_fos_password(password: &str, salt: &str, stored_hash_b64: &str) -> bool {
    let salted = if salt.is_empty() {
        password.to_owned()
    } else {
        format!("{password}{{{salt}}}")
    };
    let salted_bytes = salted.as_bytes();
    let mut digest = Sha512::digest(salted_bytes).to_vec();
    let legacy = base64::engine::general_purpose::STANDARD.encode(&digest);
    let legacy_matches = legacy.as_bytes().ct_eq(stored_hash_b64.as_bytes());
    for _ in 1..5000 {
        let mut h = Sha512::new();
        h.update(&digest);
        h.update(salted_bytes);
        digest = h.finalize().to_vec();
    }
    let computed = base64::engine::general_purpose::STANDARD.encode(&digest);
    bool::from(legacy_matches | computed.as_bytes().ct_eq(stored_hash_b64.as_bytes()))
}

/// New passwords use the previously supported 5000-iteration variant.
pub fn hash_fos_password(password: &str, salt: &str) -> String {
    let salted = if salt.is_empty() {
        password.to_owned()
    } else {
        format!("{password}{{{salt}}}")
    };
    let mut digest = Sha512::digest(salted.as_bytes()).to_vec();
    for _ in 1..5000 {
        let mut h = Sha512::new();
        h.update(&digest);
        h.update(salted.as_bytes());
        digest = h.finalize().to_vec();
    }
    base64::engine::general_purpose::STANDARD.encode(digest)
}

pub fn password_version(stored_hash: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(stored_hash.as_bytes()))
}

#[cfg(test)]
mod password_tests {
    use super::{hash_fos_password, password_version, verify_fos_password};

    #[test]
    fn password_changes_produce_new_session_versions() {
        let old = hash_fos_password("old test password", "salt-one");
        let new = hash_fos_password("new test password", "salt-two");
        assert!(verify_fos_password("new test password", "salt-two", &new));
        assert!(!verify_fos_password("old test password", "salt-two", &new));
        assert_ne!(password_version(&old), password_version(&new));
        assert_eq!(password_version(&new), password_version(&new));
    }

    #[test]
    fn accepts_original_symfony_encoder_fixtures() {
        // Generated by the encoder in the original PartKeepr container,
        // using public fixture inputs, for both supported iteration counts.
        for (salt, hash) in [
            ("fixture-salt", "ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ=="),
            ("", "56Pet5rfF3IeVtwi7PY/i/RYiPiK3LqjvHoKZxDlneP760AzH47ELwwL7DA3dlezZ7nLYA8ET6scqvGGfms9sw=="),
            ("fixture-salt", "yHgCQ7ColvqcV6tDjMXCWqG9z2iwnS04Kat228jJB5PjtOAOPnshKwxjQLYV72Jq/Z0dXbk32gGiFCxjQSNIuw=="),
            ("", "t6IMG9m1WloOJoNUlEHdxZncGEmOgYKjuZ2Cc132ouOL3jVIwlMBJ0h1Q48SzCSBR81PvCMFzI8Y0kwS/v0dDA=="),
        ] {
            assert!(verify_fos_password("migration-test-password", salt, hash));
            assert!(!verify_fos_password("incorrect-password", salt, hash));
        }
    }

    #[test]
    fn rejects_invalid_hashes() {
        for hash in ["", "invalid", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="] {
            assert!(!verify_fos_password("migration-test-password", "fixture-salt", hash));
        }
    }
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

#[derive(Debug, Serialize)]
pub struct CurrentUserView {
    pub user_id: i32,
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
    if username.is_empty() || req.password.is_empty() || req.password.len() > 1024 {
        return Err(AppError::BadRequest("username and password are required"));
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
        password_version: password_version(&fos.password),
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
    let mut cookie = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .max_age(TimeDuration::ZERO)
        .build();
    cookie.set_value("");
    (jar.remove(cookie), StatusCode::NO_CONTENT).into_response()
}

/// `GET /api/me` — return the current user. The middleware already
/// ensured authentication; we just echo the payload back.
pub async fn me(axum::Extension(user): axum::Extension<CurrentUser>) -> Json<CurrentUserView> {
    Json(CurrentUserView {
        user_id: user.user_id,
        username: user.username.clone(),
        is_admin: user.is_admin,
    })
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
    State(pool): State<MySqlPool>,
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

    // An explicitly supplied credential takes precedence: never fall back to a
    // browser cookie when a caller supplied an invalid Authorization header.
    if req.headers().contains_key(header::AUTHORIZATION) {
        let values = req.headers().get_all(header::AUTHORIZATION);
        let mut values = values.iter();
        let token = values.next().and_then(|v| v.to_str().ok()).and_then(|v| {
            let mut fields = v.split_whitespace();
            match (fields.next(), fields.next(), fields.next()) {
                (Some(scheme), Some(token), None) if scheme.eq_ignore_ascii_case("bearer") => {
                    Some(token)
                }
                _ => None,
            }
        });
        let token = match token {
            Some(token) if values.next().is_none() => token,
            _ => {
                return AppError::Unauthorized("Use Authorization: Bearer <api-key>.")
                    .into_response()
            }
        };
        let (user, read_only, key_id) = match super::api_keys::authenticate(&pool, token).await {
            Ok(identity) => identity,
            Err(error) => return error.into_response(),
        };
        if read_only && req.method() != Method::GET && req.method() != Method::HEAD {
            return AppError::Forbidden("This API key has read-only access.").into_response();
        }
        if let Err(error) = super::api_keys::record_use(&pool, key_id).await {
            return error.into_response();
        }
        req.extensions_mut().insert(AuthMethod::ApiKey);
        req.extensions_mut().insert(user);
        return next.run(req).await;
    }

    let cookie = jar.get(SESSION_COOKIE);
    match cookie.and_then(|c| signer.decode(c.value())) {
        Some(user) => {
            // Resolve the identity/status from the DB, and invalidate all
            // previous sessions when the password hash changes.
            let account: Result<Option<(String, String, bool)>, sqlx::Error> = sqlx::query_as(
                "SELECT f.password, f.username, p.admin FROM FOSUser f \
                 JOIN PartKeeprUser p ON p.username = f.username \
                 WHERE f.id = ? AND p.id = ? AND f.enabled = 1 AND f.locked = 0 AND p.active = 1",
            )
            .bind(user.fos_user_id)
            .bind(user.user_id)
            .fetch_optional(&pool)
            .await;
            let (stored, username, is_admin) = match account {
                Ok(Some(account)) => account,
                Ok(None) => return AppError::Unauthorized("Please sign in again.").into_response(),
                Err(error) => return AppError::Db(error).into_response(),
            };
            if !bool::from(
                password_version(&stored)
                    .as_bytes()
                    .ct_eq(user.password_version.as_bytes()),
            ) {
                return AppError::Unauthorized("Please sign in again.").into_response();
            }
            let user = CurrentUser {
                username,
                is_admin,
                ..user
            };
            req.extensions_mut().insert(AuthMethod::Session);
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "not authenticated"})),
        )
            .into_response(),
    }
}
