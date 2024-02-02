use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;
use sqlx::Acquire;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{auth::Auth, task::ConnectSubtask},
};

#[tracing::instrument(err)]
#[utoipa::path(post, path = super::TaskPaths::connect_subtask(), responses((status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ConnectSubtask>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
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

    // タスク同士が循環していないかを確認する。
    // payload.parent_task_idの祖先に、payload.subtask_idを持つtaskが存在しないことを確認する。
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT subtask_id, parent_task_id
            FROM subtask_connections
            WHERE subtask_id = $1

            UNION

            SELECT s.subtask_id, s.parent_task_id
            FROM subtask_connections s
            JOIN ancestors a ON s.subtask_id = a.parent_task_id
        )

        SELECT DISTINCT parent_task_id
        FROM ancestors
        WHERE parent_task_id = $2
        "#,
        payload.parent_task_id,
        payload.subtask_id
    )
    .fetch_all(&mut *conn)
    .await?;

    if !result.is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("タスクの循環は許可されていません。"),
        ));
    }

    sqlx::query!(
        "INSERT INTO subtask_connections(parent_task_id, subtask_id, user_id) VALUES($1, $2, $3);",
        payload.parent_task_id,
        payload.subtask_id,
        user.id,
    )
    .execute(&mut *conn)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::app::{tests::AppTest, AppResult, Db};
    use crate::features::task::routes::TaskPaths;
    use crate::features::task::test::task_factory::{self};
    use crate::features::task::ConnectSubtask;
    use crate::features::user::test::user_factory;

    #[sqlx::test]
    async fn サブタスクを作成できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent_task = task_factory::create_with_user(&db, &user.id).await?;
        let subtask = task_factory::create_with_user(&db, &user.id).await?;

        test.server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: parent_task.id.clone(),
                subtask_id: subtask.id.clone(),
            })
            .await;
        let fetched_subtask = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1;",
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
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: other_user_task1.id,
                subtask_id: other_user_task2.id,
            })
            .await;

        let subtasks = sqlx::query!("SELECT * FROM subtask_connections;")
            .fetch_all(&db)
            .await?;

        assert!(subtasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn サブタスク関係を相互に持たせることはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 = task_factory::create_subatsk(&db, &user.id, &task1.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: task2.id.clone(),
                subtask_id: task1.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let subtasks = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2;",
            task2.id,
            task1.id
        )
        .fetch_all(&db)
        .await?;
        assert!(subtasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn タスクを循環させることはできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 = task_factory::create_subatsk(&db, &user.id, &task1.id).await?;
        let task3 = task_factory::create_subatsk(&db, &user.id, &task2.id).await?;
        let task4 = task_factory::create_subatsk(&db, &user.id, &task3.id).await?;
        let task5 = task_factory::create_subatsk(&db, &user.id, &task4.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: task5.id.clone(),
                subtask_id: task2.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let subtasks = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2",
            task5.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(subtasks.is_empty());

        Ok(())
    }
}
