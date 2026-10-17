// SPDX-License-Identifier: GPL-3.0-or-later

//! Self-service account operations. Identity always comes from the session.
use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::auth::{self, CurrentUser};
use crate::error::AppError;

const PREFERENCE_KEY: &str = "partkeepr-ng.account";

#[derive(Serialize, sqlx::FromRow)]
pub struct Profile {
    username: String,
    email: Option<String>,
    is_admin: bool,
}

pub async fn profile(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Profile>, AppError> {
    let row = sqlx::query_as::<_, Profile>(
        "SELECT f.username, f.email, p.admin AS is_admin FROM FOSUser f \
         JOIN PartKeeprUser p ON p.username = f.username WHERE f.id = ? AND p.id = ?",
    )
    .bind(user.fos_user_id)
    .bind(user.user_id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileWrite {
    email: String,
}

fn valid_email(email: &str) -> bool {
    email.len() <= 255
        && !email.chars().any(char::is_whitespace)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.is_empty() && !domain.contains('@')
        })
}

pub async fn update_profile(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<ProfileWrite>,
) -> Result<StatusCode, AppError> {
    let email = req.email.trim();
    if !valid_email(email) {
        return Err(AppError::BadRequest(
            "Enter a valid email address (up to 255 bytes).",
        ));
    }
    let mut tx = pool.begin().await?;
    let duplicate: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM FOSUser WHERE email_canonical = ? AND id <> ?")
            .bind(email.to_lowercase())
            .bind(user.fos_user_id)
            .fetch_one(&mut *tx)
            .await?;
    if duplicate > 0 {
        return Err(AppError::BadRequest(
            "That email address is already in use.",
        ));
    }
    sqlx::query("UPDATE FOSUser SET email = ?, email_canonical = ? WHERE id = ?")
        .bind(email)
        .bind(email.to_lowercase())
        .bind(user.fos_user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE PartKeeprUser SET email = ? WHERE id = ?")
        .bind(email)
        .bind(user.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub start_section: String,
    pub parts_limit: u32,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            start_section: "categories".into(),
            parts_limit: 200,
        }
    }
}
impl Preferences {
    fn validate(&self) -> Result<(), AppError> {
        if !["categories", "storage", "footprints", "projects"]
            .contains(&self.start_section.as_str())
            || ![50, 100, 200].contains(&self.parts_limit)
        {
            return Err(AppError::BadRequest(
                "Choose a supported start section and parts limit.",
            ));
        }
        Ok(())
    }
}

pub async fn preferences(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Preferences>, AppError> {
    let stored: Option<String> = sqlx::query_scalar(
        "SELECT preferenceValue FROM UserPreference WHERE user_id = ? AND preferenceKey = ?",
    )
    .bind(user.user_id)
    .bind(PREFERENCE_KEY)
    .fetch_optional(&pool)
    .await?;
    let prefs = stored
        .as_deref()
        .and_then(|s| serde_json::from_str::<Preferences>(s).ok())
        .filter(|p| p.validate().is_ok())
        .unwrap_or_default();
    Ok(Json(prefs))
}

pub async fn update_preferences(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<Preferences>,
) -> Result<Json<Preferences>, AppError> {
    req.validate()?;
    let value = serde_json::to_string(&req)
        .map_err(|_| AppError::Internal("Could not save preferences.".into()))?;
    sqlx::query(
        "INSERT INTO UserPreference (user_id, preferenceKey, preferenceValue) VALUES (?, ?, ?) \
                 ON DUPLICATE KEY UPDATE preferenceValue = VALUES(preferenceValue)",
    )
    .bind(user.user_id)
    .bind(PREFERENCE_KEY)
    .bind(value)
    .execute(&pool)
    .await?;
    Ok(Json(req))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordWrite {
    current_password: String,
    new_password: String,
    confirm_password: String,
}
impl PasswordWrite {
    fn validate(&self) -> Result<(), AppError> {
        if self.current_password.is_empty() || self.current_password.len() > 1024 {
            return Err(AppError::BadRequest("Enter your current password."));
        }
        if self.new_password.chars().count() < 12 || self.new_password.len() > 1024 {
            return Err(AppError::BadRequest(
                "Use at least 12 characters and no more than 1024 bytes for the new password.",
            ));
        }
        if self.new_password != self.confirm_password {
            return Err(AppError::BadRequest("The new passwords do not match."));
        }
        if self.current_password == self.new_password {
            return Err(AppError::BadRequest("Choose a different new password."));
        }
        Ok(())
    }
}

pub async fn change_password(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
    jar: axum_extra::extract::cookie::CookieJar,
    Json(req): Json<PasswordWrite>,
) -> Result<axum::response::Response, AppError> {
    req.validate()?;
    // Lock while verifying, so concurrent password changes cannot both succeed
    // using the same old password. Do not include credentials in logs/errors.
    let mut tx = pool.begin().await?;
    let (stored, salt): (String, String) =
        sqlx::query_as("SELECT password, salt FROM FOSUser WHERE id = ? FOR UPDATE")
            .bind(user.fos_user_id)
            .fetch_one(&mut *tx)
            .await?;
    if !auth::verify_fos_password(&req.current_password, &salt, &stored) {
        return Err(AppError::BadRequest("The current password is incorrect."));
    }
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let hashed = auth::hash_fos_password(&req.new_password, &salt);
    sqlx::query(
        "UPDATE FOSUser SET password = ?, salt = ?, password_requested_at = NULL WHERE id = ?",
    )
    .bind(hashed)
    .bind(salt)
    .bind(user.fos_user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(auth::logout(jar).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_account_inputs() {
        assert!(valid_email("user@example.com"));
        for email in [
            "",
            "user",
            "@example.com",
            "user@",
            "user@a@b",
            "user @example.com",
        ] {
            assert!(!valid_email(email));
        }
        assert!(Preferences::default().validate().is_ok());
        assert!(Preferences {
            start_section: "unknown".into(),
            parts_limit: 200
        }
        .validate()
        .is_err());
        assert!(Preferences {
            start_section: "categories".into(),
            parts_limit: 0
        }
        .validate()
        .is_err());
        assert!(serde_json::from_str::<Preferences>(r#"{"user_id":2}"#).is_err());
        let mut req = PasswordWrite {
            current_password: "old".into(),
            new_password: "long new password".into(),
            confirm_password: "long new password".into(),
        };
        assert!(req.validate().is_ok());
        req.confirm_password = "different".into();
        assert!(req.validate().is_err());
        req.new_password = "short".into();
        req.confirm_password = "short".into();
        assert!(req.validate().is_err());
    }
}
