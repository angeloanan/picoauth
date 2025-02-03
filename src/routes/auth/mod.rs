use axum::{Router, routing::post};

use crate::AppState;

pub mod login;
pub mod register;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login::post))
        .route("/register", post(register::post))
}
