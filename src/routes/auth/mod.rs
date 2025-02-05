use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

pub mod forgot_password;
pub mod login;
pub mod register;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login::post))
        .route("/register", post(register::post))
        .route("/forgot_password/{token}", get(forgot_password::get))
        .route(
            "/forgot_password",
            post(forgot_password::post).put(forgot_password::put),
        )
}
