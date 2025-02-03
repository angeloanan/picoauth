use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use libsql::params;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, info, instrument, warn};

use crate::{
    AppState,
    common::{DATABASE_BUSY_RESPONSE, INVALID_USERNAME_PASSWORD_RESPONSE},
    db::User,
    password::verify,
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
        info!("User not found");
        return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
    };

    let db_username = user.get_str(0).unwrap();
    let db_password = user.get_str(1).unwrap();
    let db_requires_second_factor = user.get::<bool>(2).unwrap();
    info!("{user:?}");

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

    let is_password_match = verify(&password, db_password);

    if !is_password_match {
        return INVALID_USERNAME_PASSWORD_RESPONSE.clone();
    }

    return (StatusCode::OK, Json(json!({})));
}
