#[cfg(test)]
pub mod routers {
    use crate::app::AppResult;
    use crate::{
        app::AppState,
        features::{
            auth::{routers::signup::CreateUser, Auth},
            user::User,
        },
    };
    use axum::{extract::State, routing::post, Json, Router};
    use axum_login::AuthSession;
    use http::StatusCode;

    pub struct Paths;
    impl Paths {
        /// 指定したユーザーでログインセッションを作成する
        pub fn test_login() -> String {
            "/test/login".into()
        }
    }

    impl Default for CreateUser {
        fn default() -> Self {
            CreateUser {
                name: "user".into(),
                profile: "profile".into(),
            }
        }
    }

    async fn test_login_handler(
        mut auth_session: AuthSession<Auth>,
        State(AppState { db }): State<AppState>,
        Json(payload): Json<CreateUser>,
    ) -> AppResult<(StatusCode, Json<User>)> {
        let id = uuid::Uuid::new_v4().to_string();
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users(id, name, profile) VALUES($1, $2, $3) RETURNING *;",
            id,
            payload.name,
            payload.profile
        )
        .fetch_one(&db)
        .await?;

        auth_session.login(&user).await?;

        Ok((StatusCode::OK, Json(user)))
    }

    pub fn router() -> Router<AppState> {
        Router::new().route(&Paths::test_login(), post(test_login_handler))
    }
}
