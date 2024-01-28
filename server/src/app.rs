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

async fn build_inner(db: Db, router: Option<Router<AppState>>) -> Router {
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

    let app = if let Some(router) = router {
        Router::new().merge(router)
    } else {
        Router::new()
    };

    app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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

pub async fn build(db: Db) -> Router {
    build_inner(db, None).await
}

#[cfg(test)]
pub mod tests {
    use crate::{
        features::{
            auth::{self, routers::signup::CreateUser},
            user::User,
        },
        AppResult, Db,
    };
    use axum_test::TestServer;

    pub struct AppTest {
        server: TestServer,
    }
    impl AppTest {
        pub async fn new(db: &Db) -> AppResult<Self> {
            let router = super::build_inner(db.clone(), Some(auth::test::routes::router())).await;
            let mut server = TestServer::new(router)?;
            server.do_save_cookies();

            Ok(AppTest { server })
        }

        /// 指定したユーザーでログイン状態にする
        pub async fn login(&self, create_user: Option<CreateUser>) -> AppResult<User> {
            let logged_in_user: User = self
                .server
                .post(&auth::test::routes::Paths::test_login())
                .json(&create_user.unwrap_or_default())
                .await
                .json();

            Ok(logged_in_user)
        }

        pub fn server(&self) -> &TestServer {
            &self.server
        }
    }
}
