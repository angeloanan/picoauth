use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use libsql::params;
use serde_json::json;
use tracing::warn;

use crate::AppState;

pub async fn health_check(State(app): State<AppState>) -> impl IntoResponse {
    let Ok(db) = app.db.connect() else {
        warn!("Unable to connect to database!");
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    };

    let Ok(q) = db.query("PRAGMA quick_check", params![]).await else {
        warn!("Unable to execute quick database integrity check");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Unable to start database integrity check"
            })),
        )
            .into_response();
    };

    // Integrity check will return 1 row with value `ok` if everything is ok
    // Otherwise, something is wrong
    if q.column_count() != 1 {
        warn!("Database integrity check failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Database integrity check returned an error"
            })),
        )
            .into_response();
    }

    (StatusCode::NO_CONTENT).into_response()
}
