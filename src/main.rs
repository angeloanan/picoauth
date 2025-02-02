mod common;
mod db;
mod password;
mod routes;
mod totp;

use std::sync::Arc;

use axum::Router;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, normalize_path::NormalizePathLayer,
};

#[derive(Clone)]
pub struct AppState {
    db: Arc<libsql::Database>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let ct = CancellationToken::new();
    let database = db::prepare().await;
    let app_state = AppState {
        db: Arc::new(database),
    };

    let app = Router::new()
        .with_state(app_state)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new())
        .layer(
            CompressionLayer::new()
                .zstd(true)
                .quality(tower_http::CompressionLevel::Precise(19)),
        );

    let socket = tokio::net::UnixListener::bind("/var/run/picoauth.sock")
        .expect("Unable to bind to /var/run/picoauth.sock.");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind to port!");

    {
        let ct = ct.clone();
        let app = app.clone();

        tokio::spawn(async move {
            select! {
                _ = ct.cancelled() => {}
                _ = axum::serve(socket, app) => {}
            }
        });
    }

    axum::serve(listener, app).await.unwrap();

    tokio::signal::ctrl_c().await.unwrap();
    ct.cancel();
}
