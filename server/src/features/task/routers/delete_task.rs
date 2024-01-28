use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    features::{auth::Auth, task::Task},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::Paths::oas_task(), responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let task = sqlx::query_as!(
        Task,
        r#"DELETE FROM tasks WHERE id = $1 AND user_id = $2 RETURNING *;"#,
        id,
        user.id
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::features::task::test::factory as task_factory;
    use crate::features::user::test::factory as user_factory;
    use crate::{app::tests::AppTest, features::task::routers::Paths, Db};

    use super::*;

    #[sqlx::test]
    async fn 指定したタスクだけを削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        task_factory::create_with_user(&db, &user.id).await?;
        let created_task = task_factory::create_with_user(&db, &user.id).await?;

        test.server()
            .delete(&Paths::one_task(&created_task.id))
            .await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクは削除できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;

        test.login(None).await?;
        let res = test
            .server()
            .delete(&Paths::one_task(&other_user_task.id))
            .await;
        assert_ne!(res.status_code(), StatusCode::UNAUTHORIZED);

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }
}
