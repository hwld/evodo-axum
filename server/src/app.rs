use axum::Router;
use axum_login::{
    tower_sessions::{
        cookie::{time::Duration, SameSite},
        Expiry, SessionManagerLayer,
    },
    AuthManagerLayerBuilder,
};
use http::{header::CONTENT_TYPE, Method};
use tower_http::cors::CorsLayer;
use tower_sessions_sqlx_store::SqliteStore;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::{features, AppState, Db};

pub async fn build(db: Db) -> Router {
    dotenv::dotenv().expect("Failed to read .env file");

    #[utoipauto]
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    let session_store = SqliteStore::new(db.clone())
        .with_table_name("sessions")
        .expect("Failed to create session store");
    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let auth = features::auth::Auth::new(db.clone()).await;
    let auth_layer = AuthManagerLayerBuilder::new(auth, session_layer).build();

    Router::new()
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
        .with_state(AppState { db })
}

#[cfg(test)]
pub mod tests {
    use axum_test::TestServer;

    use crate::{AppResult, Db};

    pub async fn build(db: Db) -> AppResult<TestServer> {
        Ok(TestServer::new(super::build(db).await)?)
    }
}
