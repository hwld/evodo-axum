use axum::{http::StatusCode, response::IntoResponse, Router};
use axum_login::{
    tower_sessions::{
        cookie::{time::Duration, SameSite},
        Expiry, MemoryStore, SessionManagerLayer,
    },
    AuthManagerLayerBuilder,
};
use http::{header::CONTENT_TYPE, Method};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use tower_http::cors::CorsLayer;
use tracing::debug;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

mod features;

#[derive(Debug)]
struct AppError(anyhow::Error);

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

type AppResult<T> = anyhow::Result<T, AppError>;

type Db = Pool<Sqlite>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");

    tracing_subscriber::fmt::init();

    let db = SqlitePool::connect(&env::var("DATABASE_URL").expect("connect error"))
        .await
        .expect("Failed to connect");

    #[utoipauto]
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let auth = features::auth::Auth::new(db.clone()).await;
    let auth_layer = AuthManagerLayerBuilder::new(auth, session_layer).build();

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(features::auth::router())
        .merge(features::task::router())
        .merge(features::task_node::router())
        .layer(
            CorsLayer::new()
                // TODO
                .allow_origin(["http://localhost:3000".parse().unwrap()])
                .allow_credentials(true)
                .allow_headers([CONTENT_TYPE])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::DELETE,
                    Method::PUT,
                ]),
        )
        .layer(auth_layer)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("Failed to bind");

    debug!(
        "listening on {:#}",
        listener.local_addr().expect("Failed to get local_adde")
    );
    axum::serve(listener, app).await.unwrap();
}
