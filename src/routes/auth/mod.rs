use axum::{
    Router,
    routing::{get, post, put},
};

use crate::AppState;

pub mod forgot_password;
pub mod login;
pub mod logout_from_all;
pub mod me;
pub mod register;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", put(me::put))
        .route("/login", post(login::post))
        .route("/register", post(register::post))
        .route("/forgot_password", post(forgot_password::post))
        .route(
            "/forgot_password/{token}",
            get(forgot_password::get).put(forgot_password::put),
        )
}
