use std::time::{Duration, UNIX_EPOCH};

use axum::{Json, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use libsql::params;
use serde_json::{Map, Value};
use tracing::{error, instrument, warn};

use crate::{AppState, common::DATABASE_BUSY_RESPONSE, jwt};

/// Refreshes an access token (and optionally, a refresh token) using a valid refresh token
#[instrument(skip(state, authorization, body))]
pub async fn post(
    State(state): State<AppState>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    let refresh_token = if let Some(header) = authorization {
        header.token().to_string()
    } else if !body.is_empty() {
        body
    } else {
        return (StatusCode::UNAUTHORIZED).into_response();
    };

    let Ok(conn) = state.db.connect() else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    // Check if JWT has been revoked
    let Ok(mut revoked_jwt_query) = conn
        .query(
            "SELECT 1 FROM \"revoked_jwt\" WHERE token = ?",
            params![refresh_token.clone()],
        )
        .await
    else {
        warn!("Unable to query for revoked JWT!");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(revoked_jwt_row) = revoked_jwt_query.next().await else {
        warn!("Unable to query for revoked JWT!");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    if revoked_jwt_row.is_some() {
        // Revoked token found
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    let Ok(token_data) = jwt::verify_refresh_token(&refresh_token) else {
        // Invalid refresh token
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    let claims = token_data.claims;
    let Ok(user_id) = u64::from_str_radix(&claims.sub, 10) else {
        error!(
            "Signed Refresh token contains an invalid user ID! Unless frontend is provided the same JWT key, JWT key has been compromised!"
        );
        return (StatusCode::UNAUTHORIZED).into_response();
    };

    // Query for up-to-date user data
    let Ok(mut query) = conn
        .query(
            "SELECT username, display_name, email, email_verified_at FROM \"users\" WHERE id = ?",
            params![user_id],
        )
        .await
    else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };
    let Ok(user) = query.next().await else {
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Some(user) = user else {
        // May be invalid if user has been deleted off of database
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    let db_username = user.get::<String>(0).unwrap();
    let db_display_name = user.get::<Option<String>>(1).unwrap();
    let db_email = user.get::<Option<String>>(2).unwrap();
    let db_email_verified = user.get::<Option<i64>>(3).unwrap();
    let email_verified = match db_email_verified {
        Some(_) => Some(true),
        None if db_email.is_some() => Some(false),
        None => None,
    };

    let access_token = jwt::issue_access_token(
        user_id,
        &db_username,
        db_display_name.as_deref(),
        db_email.as_deref(),
        email_verified,
        claims.auth_time,
    )
    .to_string();
    let mut response_data = Map::new();
    response_data.insert("access_token".to_string(), Value::String(access_token));

    let current_time = UNIX_EPOCH.elapsed().unwrap().as_secs();
    let refresh_token_expirity_time = claims.exp as u64;

    // Generate & include new refresh token if token will expire in less than 1 day
    if current_time > refresh_token_expirity_time - Duration::from_secs(24 * 3600).as_secs() {
        let new_refresh_token =
            jwt::issue_refresh_token(user_id, Some(claims.auth_time)).to_string();
        response_data.insert(
            "refresh_token".to_string(),
            Value::String(new_refresh_token),
        );
    }

    // All guards / checks above
    // Renew JWT
    (StatusCode::OK, Json(response_data)).into_response()
}
