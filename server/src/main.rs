use axum::{http::StatusCode, response::IntoResponse};
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::debug;
mod app;
mod config;
mod features;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type AppResult<T> = anyhow::Result<T, AppError>;

type Db = Pool<Sqlite>;

#[derive(Debug, Clone)]
pub struct AppState {
    db: Db,
}

impl axum::extract::FromRef<AppState> for () {
    fn from_ref(_: &AppState) {}
}
#[tokio::main]
async fn main() {
    config::load();

    tracing_subscriber::fmt::init();

    let db = SqlitePool::connect(&config::database_url().expect("Failed load database_url"))
        .await
        .expect("Failed to connect");

    let app = app::build(db).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("Failed to bind");

    debug!(
        "listening on {:#}",
        listener.local_addr().expect("Failed to get local_adde")
    );
    axum::serve(listener, app).await.unwrap();
}
