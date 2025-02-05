use axum::{Router, routing::get};

use crate::AppState;

pub mod user;
pub mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(users::get).post(users::post))
        .route(
            "/user/{user_id}",
            get(user::get).put(user::put).delete(user::delete),
        )
}
