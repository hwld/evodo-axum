use axum::{extract::State, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{db::detect_circular_connection, ConnectSubtask, ReconnectSubtask},
    },
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::TaskPaths::reconnect_subtask(), responses((status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ReconnectSubtask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    sqlx::query!(
        "DELETE FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2 AND user_id = $3 RETURNING *",
        payload.old_parent_task_id,
        payload.old_subtask_id,
        user.id
    ).fetch_one(&mut *tx).await?;

    if detect_circular_connection(
        &mut tx,
        ConnectSubtask {
            parent_task_id: payload.new_parent_task_id.clone(),
            subtask_id: payload.new_subtask_id.clone(),
        },
    )
    .await?
    {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("タスクの循環は許可されていません。"),
        ));
    }

    sqlx::query!(
        "INSERT INTO subtask_connections(parent_task_id, subtask_id, user_id) VALUES($1, $2, $3) RETURNING *;",
        payload.new_parent_task_id,
        payload.new_subtask_id,
        user.id
    ).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            task::{routes::TaskPaths, test::task_factory, ReconnectSubtask},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn サブタスクの再接続ができる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 = task_factory::create_subatsk(&db, &user.id, &task1.id.clone()).await?;
        let task3 = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::reconnect_subtask())
            .json(&ReconnectSubtask {
                old_parent_task_id: task1.id.clone(),
                old_subtask_id: task2.id.clone(),
                new_parent_task_id: task2.id.clone(),
                new_subtask_id: task3.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let deleted = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2;",
            task1.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(deleted.is_empty());

        let created = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2",
            task2.id,
            task3.id
        )
        .fetch_all(&db)
        .await?;
        assert!(!created.is_empty());

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

        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ReconnectSubtask {
                old_parent_task_id: task3.id.clone(),
                old_subtask_id: task4.id.clone(),
                new_parent_task_id: task3.id.clone(),
                new_subtask_id: task2.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let old = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id= $1 AND subtask_id = $2",
            task3.id,
            task4.id
        )
        .fetch_all(&db)
        .await?;
        assert_eq!(old.len(), 1);

        let subtasks = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2",
            task3.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(subtasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 他の人のサブタスク関係を更新することはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_task1 = task_factory::create_with_user(&db, &other_user.id).await?;
        let other_task2 =
            task_factory::create_subatsk(&db, &other_user.id, &other_task1.id).await?;

        let user = test.login(None).await?;
        let my_task = task_factory::create_with_user(&db, &user.id).await?;
        let res = test
            .server()
            .put(&TaskPaths::reconnect_subtask())
            .json(&ReconnectSubtask {
                old_parent_task_id: other_task1.id.clone(),
                old_subtask_id: other_task2.id.clone(),
                new_parent_task_id: other_task2.id.clone(),
                new_subtask_id: my_task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let old_connection = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2 AND user_id = $3",
            other_task1.id,
            other_task2.id,
            other_user.id
        ).fetch_all(&db).await?;
        assert_eq!(old_connection.len(), 1);

        let new_connection = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2 AND user_id = $3",
            other_task2.id,
            my_task.id,
            user.id
        ).fetch_all(&db).await?;
        assert!(new_connection.is_empty());

        Ok(())
    }
}
