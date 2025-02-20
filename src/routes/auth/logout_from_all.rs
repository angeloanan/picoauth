use axum::{http::StatusCode, response::IntoResponse};

pub async fn post() -> impl IntoResponse {
    // TODO: Add to database & refactor jwt::validate
    (StatusCode::NO_CONTENT).into_response()
}
