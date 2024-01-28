use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    features::{auth::Auth, task::Task},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::Paths::tasks(), responses((status = 200, body = [Task])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let tasks = sqlx::query_as!(Task, r#"SELECT * FROM tasks WHERE user_id = $1;"#, user.id)
        .fetch_all(&db)
        .await?;

    Ok((StatusCode::OK, Json(tasks)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::tests::AppTest,
        features::{
            task::{self, routers::Paths},
            user::{self, User},
        },
        Db,
    };

    use super::*;

    #[sqlx::test]
    async fn 自分の全てのタスクを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user::test::factory::create(&db, Some(User::default())).await?;
        task::test::factory::create(&db, other_user.id, None).await?;

        let user = test.login(None).await?;
        tokio::try_join!(
            task::test::factory::create(&db, user.clone().id, None),
            task::test::factory::create(&db, user.clone().id, None),
            task::test::factory::create(&db, user.clone().id, None),
        )?;

        let tasks: Vec<Task> = test.server().get(&Paths::tasks()).await.json();
        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
