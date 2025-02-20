#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]

mod common;
mod db;
mod jwt;
mod password;
mod routes;
mod totp;

use std::{sync::Arc, time::Duration};

use axum::{Router, routing::get};
use common::RequestIdCounter;
use routes::health_check::health_check;
use tokio::{select, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    normalize_path::NormalizePathLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::{Timeout, TimeoutLayer},
};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: Arc<libsql::Database>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Only load .env file on debug builds
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    // Verify proper envvar
    std::env::var("JWT_SECRET").expect("No JWT_SECRET provided!");

    let ct = CancellationToken::new();
    let database = db::prepare().await;
    let mut http_servers: JoinSet<()> = JoinSet::new();

    let app_state = AppState {
        db: Arc::new(database),
    };

    let mut app = Router::new()
        .nest("/auth", routes::auth::router())
        .nest("/jwt", routes::jwt::router())
        .route("/health-check", get(health_check))
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::x_request_id(RequestIdCounter::default()))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .with_state(app_state);

    if std::env::var("HTTP_COMPRESS").is_ok() {
        info!("Env var `HTTP_COMPRESS` is set. Compressing responses...");
        app = app.layer(
            CompressionLayer::new()
                .zstd(true)
                .quality(tower_http::CompressionLevel::Precise(19)),
        );
    }

    #[cfg(not(debug_assertions))]
    let socket = tokio::net::UnixListener::bind("/var/run/picoauth.sock")
        .expect("Unable to bind to /var/run/picoauth.sock.");
    #[cfg(debug_assertions)]
    let socket = tokio::net::UnixListener::bind("./picoauth.sock")
        .expect("Unable to bind to ./picoauth.sock");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind to port!");

    // Spawn Unix socket server
    {
        let ct = ct.clone();
        let app = app.clone();

        info!("Listening on ./picoauth.sock");
        http_servers.spawn(async move {
            select! {
                _ = ct.cancelled() => {
                    info!("Caught exit signal - Shutting down Unix Socket server");
                }
                _ = axum::serve(socket, app) => {}
            }
        });
    }

    // Spawn TCP server
    {
        let ct = ct.clone();
        let app = app.clone();

        info!("Listening on http://0.0.0.0:3000");
        http_servers.spawn(async move {
            select! {
                _ = ct.cancelled() => {
                    info!("Caught exit signal - Shutting down TCP server");
                }
                _ = axum::serve(listener, app) => {}
            }
        });
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Unable to listen for SIGINT");

    // After exit signal has been caught

    info!("Quit signal captured. Shutting down gracefully...");
    ct.cancel();
    http_servers.join_all().await;

    #[cfg(not(debug_assertions))]
    std::fs::remove_file("/var/run/picoauth.sock").ok();
    #[cfg(debug_assertions)]
    std::fs::remove_file("./picoauth.sock").ok();

    info!("All HTTP servers shut down. Goodbye 👋");
}
