use axum::{Router, routing::post};

use crate::AppState;

pub mod refresh;
pub mod validate;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate::post))
        .route("/refresh", post(refresh::post))
}
