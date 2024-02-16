use axum::Router;
use axum_login::{
    tower_sessions::{
        cookie::{time::Duration, SameSite},
        Expiry, SessionManagerLayer,
    },
    AuthManagerLayerBuilder,
};
use http::{header::CONTENT_TYPE, Method};
use sqlx::{Pool, Sqlite, SqliteConnection};
use tower_http::cors::CorsLayer;
use tower_sessions_sqlx_store::SqliteStore;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::{config::Env, error::AppError, features};

pub type Db = Pool<Sqlite>;
pub type Connection = SqliteConnection;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Db,
}

impl axum::extract::FromRef<AppState> for () {
    fn from_ref(_: &AppState) {}
}

pub type AppResult<T> = anyhow::Result<T, AppError>;

async fn build_inner(db: Db, router: Option<Router<AppState>>) -> Router {
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
        .merge(features::sub_task::router())
        .merge(features::block_task::router())
        .merge(features::task_node::router())
        .layer(
            CorsLayer::new()
                .allow_origin([Env::client_url().parse().unwrap()])
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
    use crate::features::auth::routes::signup::CreateUser;
    use crate::{
        config::Env,
        features::{auth, user::User},
    };
    use axum_test::TestServer;

    use super::{AppResult, Db};

    pub struct AppTest {
        server: TestServer,
    }
    impl AppTest {
        pub async fn new(db: &Db) -> AppResult<Self> {
            Env::load();

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
