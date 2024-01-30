use sqlx::SqlitePool;
use tracing::debug;

use crate::config::Env;
mod app;
mod config;
mod error;
mod features;

#[tokio::main]
async fn main() {
    Env::load();
    tracing_subscriber::fmt::init();

    let db = SqlitePool::connect(&Env::database_url())
        .await
        .expect("Failed to connect");

    let app = app::build(db).await;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", Env::port()))
        .await
        .expect("Failed to bind");

    debug!(
        "listening on {:#}",
        listener.local_addr().expect("Failed to get local_adde")
    );
    axum::serve(listener, app).await.unwrap();
}
