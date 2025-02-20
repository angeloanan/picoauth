use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use libsql::params;
use tracing::{instrument, warn};

use crate::{AppState, common::DATABASE_BUSY_RESPONSE, jwt};

#[instrument(skip(state, authorization, body))]
pub async fn post(
    State(state): State<AppState>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    let access_token = if let Some(header) = authorization {
        header.token().to_string()
    } else if !body.is_empty() {
        body
    } else {
        return (StatusCode::UNAUTHORIZED).into_response();
    };

    let Ok(conn) = state.db.connect() else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut query) = conn
        .query(
            "SELECT 1 FROM \"revoked_jwt\" WHERE token = ?",
            params![access_token.clone()],
        )
        .await
    else {
        warn!("Unable to query for revoked JWT!");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(row) = query.next().await else {
        warn!("Unable to query for revoked JWT!");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    if row.is_some() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Ok(token_data) = jwt::verify_access_token(&access_token) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // TODO: Validate on last_logout_all time
    // if token_data.claims.auth_time

    StatusCode::NO_CONTENT.into_response()
}
