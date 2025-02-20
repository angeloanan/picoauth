use std::time::UNIX_EPOCH;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use libsql::params;
use serde::Deserialize;
use serde_json::json;
use tracing::{instrument, warn};

use crate::{
    AppState,
    common::{DATABASE_BUSY_RESPONSE, INVALID_USERNAME_PASSWORD_RESPONSE},
    jwt,
    password::verify,
    totp,
};

#[derive(Deserialize)]
pub struct UserLoginDto {
    username: String,
    password: String,
    totp: Option<String>,
}

#[instrument(skip(state, dto), fields(username = %dto.username))]
pub async fn post(
    State(state): State<AppState>,
    Json(dto): Json<UserLoginDto>,
) -> impl IntoResponse {
    let db = state.db.connect().unwrap();

    let username = dto.username;
    let password = dto.password;
    let totp = dto.totp;

    // Select username case-insensitively
    let Ok(mut query) = db
        .query(
            "SELECT username, password, requires_second_factor FROM \"users\" WHERE username = ? COLLATE NOCASE",
            params![username.clone()],
        )
        .await
    else {
        warn!("Database query failed!");
        return DATABASE_BUSY_RESPONSE.clone();
    };

    // Get first user
    let Ok(user) = query.next().await else {
        warn!("Database query failed!");
        return DATABASE_BUSY_RESPONSE.clone();
    };

    let Some(user) = user else {
        return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
    };

    // Get user fields first
    // LibSQL bug on getting previous Row on an advanced Rows - https://github.com/tursodatabase/libsql/issues/1947
    let db_username = user.get::<String>(0).unwrap();
    let db_password = user.get::<String>(1).unwrap();
    let db_requires_second_factor = user.get::<bool>(2).unwrap();

    // Sanity-check: Ensure that there is only one user with the given username
    // Shouldn't happen, may be removed in the future
    if let Ok(next_user) = query.next().await {
        if next_user.is_some() {
            warn!(
                username,
                "There seem to be multiple users with the same username. Treating as if user does not exist at all",
            );
            return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
        }
    } else {
        return DATABASE_BUSY_RESPONSE.clone();
    }

    let is_password_match = tokio::task::spawn_blocking(move || verify(&password, &db_password))
        .await
        .unwrap();

    if !is_password_match {
        return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
    }

    // Error if TOTP is given but no second factor is required
    // TODO: Is it logical to have TOTP without 2FA?
    if !db_requires_second_factor && totp.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "TOTP is provided while 2FA is not required" })),
        );
    }

    if db_requires_second_factor {
        let Some(totp) = totp else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "2FA is required" })),
            );
        };

        let Ok(mut query) = db
            .query(
                "SELECT totp_secret FROM \"users\" WHERE username = ? COLLATE NOCASE",
                params![username.clone()],
            )
            .await
        else {
            warn!("Database query failed!");
            return DATABASE_BUSY_RESPONSE.clone();
        };

        // REFACTOR: Handle edge cases & return DATABASE_BUSY_RESPONSE on error
        let totp_secret = query
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<String>(0)
            .unwrap();

        let is_totp_valid = totp::check_current(totp_secret.as_bytes(), &totp);
        if !is_totp_valid {
            return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
        }
    }

    // -------------------------
    // If everything is correct
    // Start generating JWT
    let Ok(mut query) = db
        .query(
            "SELECT id, display_name, email, email_verified_at FROM \"users\" WHERE username = ? COLLATE NOCASE",
            params![username.clone()],
        )
        .await
    else {
        warn!("Database query failed!");
        return DATABASE_BUSY_RESPONSE.clone();
    };

    let user = query.next().await.unwrap().unwrap();
    let db_userid = user.get::<u64>(0).unwrap();
    let db_display_name = user.get::<Option<String>>(1).unwrap();
    let db_email = user.get::<Option<String>>(2).unwrap();
    let db_email_verified = user.get::<Option<i64>>(3).unwrap();

    let email_verified = match db_email_verified {
        Some(_) => Some(true),
        None if db_email.is_some() => Some(false),
        None => None,
    };

    let current_time = UNIX_EPOCH.elapsed().unwrap().as_secs() as usize;
    let refresh_token = jwt::issue_refresh_token(db_userid, Some(current_time));
    let access_token = jwt::issue_access_token(
        db_userid,
        &db_username,
        db_display_name.as_deref(),
        db_email.as_deref(),
        email_verified,
        current_time,
    );

    return (
        StatusCode::OK,
        Json(json!({
            "refresh_token": refresh_token,
            "access_token": access_token,
        })),
    );
}
