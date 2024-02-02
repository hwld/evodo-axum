use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;
use sqlx::Acquire;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{auth::Auth, task::CreateSubtask},
};

#[tracing::instrument(err)]
#[utoipa::path(post, path = super::Paths::create_subtask(), responses((status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<CreateSubtask>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, None));
    };

    let mut tx = db.begin().await?;
    let conn = tx.acquire().await?;

    // ログインユーザーが指定されたタスクを持っているかを確認する
    let tasks = sqlx::query!(
        "SELECT * FROM tasks WHERE id IN ($1, $2) AND user_id = $3;",
        payload.parent_task_id,
        payload.subtask_id,
        user.id,
    )
    .fetch_all(&mut *conn)
    .await?;

    // - 他人のユーザーのタスクではないか
    // - 同じタスクIdが２つ指定されたこと
    // を検出できる
    if tasks.len() != 2 {
        return Err(AppError::new(StatusCode::NOT_FOUND, None));
    }

    sqlx::query!(
        "INSERT INTO subtasks(parent_task_id, subtask_id) VALUES($1, $2);",
        payload.parent_task_id,
        payload.subtask_id
    )
    .execute(&mut *conn)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::app::{tests::AppTest, AppResult, Db};
    use crate::features::task::routes::Paths as TaskPaths;
    use crate::features::task::test::factory as task_factory;
    use crate::features::task::CreateSubtask;
    use crate::features::user::test::factory as user_factory;

    #[sqlx::test]
    async fn サブタスクを作成できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent_task = task_factory::create_with_user(&db, &user.id).await?;
        let subtask = task_factory::create_with_user(&db, &user.id).await?;

        test.server()
            .post(&TaskPaths::create_subtask())
            .json(&CreateSubtask {
                parent_task_id: parent_task.id.clone(),
                subtask_id: subtask.id.clone(),
            })
            .await;
        let fetched_subtask = sqlx::query!(
            "SELECT * FROM subtasks WHERE parent_task_id = $1;",
            parent_task.id
        )
        .fetch_one(&db)
        .await?;

        assert_eq!(subtask.id, fetched_subtask.subtask_id);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のユーザーのタスクを指定できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task1 = task_factory::create_with_user(&db, &other_user.id).await?;
        let other_user_task2 = task_factory::create_with_user(&db, &other_user.id).await?;

        test.login(None).await?;
        test.server()
            .post(&TaskPaths::create_subtask())
            .json(&CreateSubtask {
                parent_task_id: other_user_task1.id,
                subtask_id: other_user_task2.id,
            })
            .await;

        let subtasks = sqlx::query!("SELECT * FROM subtasks;")
            .fetch_all(&db)
            .await?;

        assert_eq!(0, subtasks.len());

        Ok(())
    }
}
