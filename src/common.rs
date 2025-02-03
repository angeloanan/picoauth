use std::sync::LazyLock;

use axum::{Json, http::StatusCode};
use regex::Regex;
use serde_json::json;

pub static USERNAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,32}$").unwrap());

pub static DATABASE_BUSY_RESPONSE: LazyLock<(
    axum::http::StatusCode,
    axum::Json<serde_json::Value>,
)> = LazyLock::new(|| {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(
            json!({ "error": "Database is currently busy - please try again in a couple of seconds" }),
        ),
    )
});

pub static INVALID_USERNAME_PASSWORD_RESPONSE: LazyLock<(
    axum::http::StatusCode,
    axum::Json<serde_json::Value>,
)> = LazyLock::new(|| {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "Invalid username or password" })),
    )
});
