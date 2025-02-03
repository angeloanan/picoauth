use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use libsql::params;
use serde::Deserialize;
use serde_json::json;
use tracing::{instrument, warn};

use crate::{AppState, common::USERNAME_REGEX, password};

#[derive(Deserialize)]
pub struct RegisterUserDto {
    username: String,
    password: String,

    email: Option<String>,
    display_name: Option<String>,
}

#[instrument(skip(state, dto))]
pub async fn post(
    State(state): State<AppState>,
    Json(dto): Json<RegisterUserDto>,
) -> impl IntoResponse {
    let username = dto.username;
    let password = dto.password;

    if !USERNAME_REGEX.is_match(&username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "error": "Invalid username - Username may only alphanumeric characters, dashes (-) and underscores (_)" }),
            ),
        );
    }
    if password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Insecure password - Password must be 8 characters or longer" })),
        );
    }

    let Ok(db) = state.db.connect() else {
        warn!("Unable to connect to database");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database is busy - Please try again in a couple of seconds" })),
        );
    };
    // TODO: HIBP password check?

    // Check for duplicate username
    let mut username_count = db
        .query(
            "SELECT COUNT(*) FROM \"users\" WHERE username = ? COLLATE NOCASE",
            params![username.as_str()],
        )
        .await
        .expect("Unable to query database");
    let username_count = username_count.next().await.unwrap().unwrap();
    if username_count.get::<u32>(0).unwrap() > 0 {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Username already taken" })),
        );
    }

    // TODO: Email verification

    let password_hash = tokio::task::spawn_blocking(move || password::hash(&password))
        .await
        .unwrap();

    // Insert user into database
    let insert_result = db
        .execute(
            "INSERT INTO \"users\" (username, password, email, display_name) VALUES (?, ?, ?, ?)",
            params![
                username.as_str(),
                password_hash,
                dto.email,
                dto.display_name
            ],
        )
        .await;

    match insert_result {
        Ok(_) => {
            return (StatusCode::OK, Json(json!({ "success": true })));
        }

        Err(e) => {
            warn!("Unable to insert user into database: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({ "error": "Database is busy - Please try again in a couple of seconds" }),
                ),
            );
        }
    }
}
