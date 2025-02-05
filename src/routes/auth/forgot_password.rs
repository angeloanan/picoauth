// Considerations to be made:
// * Message should be consistent for existent / non-existent accounts
// * Time taken for the operation should be consistent - For now, we may use a fixed time configurable via environment variable
//   * Careful with queueing side-channels
// * This is user-facing endpoint; Resetting password will require a side-channel (email, SMS, etc.)
// * Going to store URL tokens with expiry in the database
//   * Tokens MUST be long & crypto safe
// TODO: MFA Should be enforced - think of how to handle this

// Frontend notes
// * Frontend SHOULD handle rate-limiting mechanism

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use libsql::{TransactionBehavior, params};
use rand::{distr, prelude::*};
use serde::Deserialize;
use tracing::{instrument, warn};

use crate::{AppState, common::DATABASE_BUSY_RESPONSE, password};

/// Query if URL token is valid
/// Does not need to contain deadline as user will be probing for a CSPRNG generated token
pub async fn get(State(state): State<AppState>, Path(token): Path<String>) -> impl IntoResponse {
    let Ok(conn) = state.db.connect() else {
        warn!("Unable to connect to the database");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut rows) = conn
        .query(
            "SELECT expires_at, used_at FROM \"forgot_password_token\" WHERE token = ?",
            params![token],
        )
        .await
    else {
        warn!("Unable to query for token existence");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(row) = rows.next().await else {
        warn!("Unable to query for token existence");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    // Extract row otherwise return token doesn't exist
    let Some(row) = row else {
        return (StatusCode::NOT_FOUND).into_response();
    };

    // Check if token has been used
    if !row.get_value(1).unwrap().is_null() {
        return (StatusCode::GONE).into_response();
    }

    // Check if token is expired
    let expirity_time = UNIX_EPOCH + Duration::from_secs(row.get::<u64>(0).unwrap());
    if SystemTime::now() > expirity_time {
        return (StatusCode::GONE).into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

// 24 hour
const FORGOT_PASSWORD_TOKEN_EXPIRY_DURATION: Duration = Duration::from_secs(24 * 3600);

#[derive(Deserialize)]
pub struct ForgotPasswordSubmitDto {
    username: String,
}

/// Submit a new forgot password request
#[instrument(skip(state, req))]
pub async fn post(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordSubmitDto>,
) -> impl IntoResponse {
    let minimum_time = std::env::var("FORGOT_PASSWORD_MINIMUM_TIME")
        .map_or(Duration::from_millis(200), |t| {
            Duration::from_millis(t.parse().unwrap())
        });
    let deadline = Instant::now() + minimum_time;

    // Process
    let Ok(conn) = state.db.connect() else {
        warn!("Unable to connect to the database");
        tokio::time::sleep_until(deadline.into()).await;
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut rows) = conn
        .query("SELECT id FROM \"users\" WHERE username = ?", params![
            req.username.clone()
        ])
        .await
    else {
        warn!("Unable to query for user existence");
        tokio::time::sleep_until(deadline.into()).await;
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(row) = rows.next().await else {
        warn!("Unable to query for user existence");
        tokio::time::sleep_until(deadline.into()).await;
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    // If user exists, do whole processing, otherwise skip this whole block
    if let Some(row) = row {
        let user_id = row.get::<u64>(0).unwrap();

        // Generate token
        // Initialize new RNG every time function gets called
        // This hopefully ensures that forward secrecy is maintained
        let rng = rand::rngs::StdRng::from_os_rng();
        let token: String = rng
            .sample_iter(distr::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let expire_time = UNIX_EPOCH.elapsed().unwrap().as_secs()
            + FORGOT_PASSWORD_TOKEN_EXPIRY_DURATION.as_secs();

        // Store token in database
        if let Err(e) = conn.execute(
            "INSERT INTO \"forgot_password_token\" (token, user_id, expires_at) VALUES (?, ?, ?)",
            params![token, user_id, expire_time],
        )
        .await {
            warn!("Unable to store password token in database, {e}");
            tokio::time::sleep_until(deadline.into()).await;
            return DATABASE_BUSY_RESPONSE.clone().into_response();
        }

        // TODO: Enqueue side-channel message
    }

    // Wait until minimum time has passed
    tokio::time::sleep_until(deadline.into()).await;

    // Send response
    (StatusCode::NO_CONTENT).into_response()
}

#[derive(Deserialize)]
pub struct ForgotPasswordExecuteDto {
    token: String,
    password: String,
}

/// Executes the forgot password request
/// Does not need to contain deadline as user will be probing for a CSPRNG generated token
pub async fn put(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordExecuteDto>,
) -> impl IntoResponse {
    let Ok(conn) = state.db.connect() else {
        warn!("Unable to connect to the database");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(txn) = conn
        .transaction_with_behavior(TransactionBehavior::Deferred)
        .await
    else {
        warn!("Unable to initialize a transaction");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(mut rows) = txn
        .query(
            "SELECT user_id, expires_at, used_at FROM \"forgot_password_token\" WHERE token = ?",
            params![req.token.clone()],
        )
        .await
    else {
        txn.rollback().await.ok();
        warn!("Unable to query for token existence");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Ok(row) = rows.next().await else {
        txn.rollback().await.ok();
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    };

    let Some(row) = row else {
        txn.rollback().await.ok();
        return (StatusCode::NOT_FOUND).into_response();
    };

    let user_id = row.get::<u64>(0).unwrap();

    // Check if token has been used
    if !row.get_value(1).unwrap().is_null() {
        txn.rollback().await.ok();
        return (StatusCode::GONE).into_response();
    }

    // Check if token is expired
    let expirity_time = UNIX_EPOCH + Duration::from_secs(row.get::<u64>(0).unwrap());
    if SystemTime::now() > expirity_time {
        txn.rollback().await.ok();
        return (StatusCode::GONE).into_response();
    }

    // --------------
    // Update password
    // Hash password
    let password_hash = tokio::task::spawn_blocking(move || password::hash(&req.password))
        .await
        .unwrap();

    // Update user password
    if let Err(e) = txn
        .execute("UPDATE \"users\" SET password = ? WHERE id = ?", params![
            password_hash,
            user_id
        ])
        .await
    {
        txn.rollback().await.ok();
        warn!("Unable to update user password, {e}");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    }

    // Mark token as used
    if let Err(e) = txn
        .execute(
            "UPDATE \"forgot_password_token\" SET used_at = ? WHERE token = ?",
            params![UNIX_EPOCH.elapsed().unwrap().as_secs(), req.token],
        )
        .await
    {
        txn.rollback().await.ok();
        warn!("Unable to mark token as used, {e}");
        return DATABASE_BUSY_RESPONSE.clone().into_response();
    }

    if let Ok(_) = txn.commit().await {
        StatusCode::OK.into_response()
    } else {
        warn!("Unable to commit transaction");
        DATABASE_BUSY_RESPONSE.clone().into_response()
    }
}
