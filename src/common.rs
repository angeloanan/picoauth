use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU64, Ordering},
};

use axum::{Json, extract::Request, http::StatusCode};
use regex::Regex;
use serde_json::json;
use tower_http::request_id::{MakeRequestId, RequestId};

#[derive(Clone, Default)]
pub struct RequestIdCounter {
    inner: Arc<AtomicU64>,
}

impl MakeRequestId for RequestIdCounter {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .inner
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

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
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Invalid username or password" })),
    )
});
