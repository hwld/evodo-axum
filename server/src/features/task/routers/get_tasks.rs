use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::{
    app::AppState,
    error::AppError,
    features::{auth::Auth, task::Task},
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::Paths::tasks(), responses((status = 200, body = [Task])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, None));
    };

    let tasks = sqlx::query_as!(Task, r#"SELECT * FROM tasks WHERE user_id = $1;"#, user.id)
        .fetch_all(&db)
        .await?;

    Ok((StatusCode::OK, Json(tasks)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::app::tests::AppTest;
    use crate::app::Db;
    use crate::features::task::routers::Paths;
    use crate::features::{
        task::test::factory as task_factory, user::test::factory as user_factory,
    };

    use super::*;

    #[sqlx::test]
    async fn 自分の全てのタスクを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        task_factory::create_with_user(&db, &other_user.id).await?;

        let user = test.login(None).await?;
        tokio::try_join!(
            task_factory::create_with_user(&db, &user.id),
            task_factory::create_with_user(&db, &user.id),
            task_factory::create_with_user(&db, &user.id),
        )?;

        let tasks: Vec<Task> = test.server().get(&Paths::tasks()).await.json();
        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
