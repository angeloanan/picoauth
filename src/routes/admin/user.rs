use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use libsql::params;
use serde_json::{Map, Value};
use tracing::{error, instrument, warn};

use crate::{AppState, common::DATABASE_BUSY_RESPONSE};

#[instrument(skip(state))]
pub async fn get(State(state): State<AppState>, Path(user_id): Path<u64>) -> impl IntoResponse {
    let Ok(conn) = state.db.connect() else {
        error!("Unable to connect to database");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut q) = conn
        .query(
            "SELECT id, username, display_name, email FROM \"users\" WHERE id = ?",
            params![user_id],
        )
        .await
    else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    if q.column_count() == 0 {
        return (StatusCode::NOT_FOUND).into_response();
    }

    let Ok(Some(user)) = q.next().await else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };
    let mut data = Map::new();
    data.insert(
        "id".to_string(),
        Value::Number(serde_json::Number::from_u128(user.get::<u64>(0).unwrap().into()).unwrap()),
    );
    data.insert(
        "username".to_string(),
        Value::String(user.get::<String>(1).unwrap()),
    );

    let display_name = user.get::<Option<String>>(2).unwrap();
    data.insert(
        "display_name".to_string(),
        display_name.map_or(Value::Null, Value::String),
    );

    let email = user.get::<Option<String>>(3).unwrap();
    data.insert(
        "email".to_string(),
        email.map_or(Value::Null, Value::String),
    );

    (StatusCode::OK, Json(data)).into_response()
}

/// Update user
#[instrument(skip(state))]
pub async fn put(State(state): State<AppState>, Path(user_id): Path<u64>) -> impl IntoResponse {}

/// Delete user
#[instrument(skip(state))]
pub async fn delete(State(state): State<AppState>, Path(user_id): Path<u64>) -> impl IntoResponse {
    let Ok(conn) = state.db.connect() else {
        error!("Unable to connect to database");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut q) = conn
        .query(
            "DELETE FROM \"users\" WHERE id = ? RETURNING id, username, display_name, email",
            params![user_id],
        )
        .await
    else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(Some(user)) = q.next().await else {
        warn!("User deleted but unable to fetch returned user data");
        return (StatusCode::NO_CONTENT).into_response();
    };

    let mut data = Map::new();
    data.insert(
        "id".to_string(),
        Value::Number(serde_json::Number::from_u128(user.get::<u64>(0).unwrap().into()).unwrap()),
    );
    data.insert(
        "username".to_string(),
        Value::String(user.get::<String>(1).unwrap()),
    );

    let display_name = user.get::<Option<String>>(2).unwrap();
    data.insert(
        "display_name".to_string(),
        display_name.map_or(Value::Null, Value::String),
    );

    let email = user.get::<Option<String>>(3).unwrap();
    data.insert(
        "email".to_string(),
        email.map_or(Value::Null, Value::String),
    );

    (StatusCode::NO_CONTENT, Json(data)).into_response()
}
