use axum::{extract::State, Json};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{auth::Auth, task::DisconnectSubtask},
};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::TaskPaths::disconnect_subtask(), responses(( status = 200)))]
pub async fn handle(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<DisconnectSubtask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    // 削除できなかったときにエラーにするためにRETURNINGとfetch_oneを使用する
    sqlx::query!(
        "DELETE FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2 AND user_id = $3 RETURNING *;",
        payload.parent_task_id,
        payload.subtask_id,
        user.id
    ).fetch_one(&db).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            task::{routes::TaskPaths, test::task_factory, DisconnectSubtask},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn サブタスク関係を削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create_with_user(&db, &user.id).await?;
        let subtask = task_factory::create_subtask(&db, &user.id, &task.id).await?;

        let res = test
            .server()
            .delete(&TaskPaths::disconnect_subtask())
            .json(&DisconnectSubtask {
                parent_task_id: task.id,
                subtask_id: subtask.id,
            })
            .await;
        res.assert_status_ok();

        let subtasks = sqlx::query!("SELECT * FROM subtask_connections;")
            .fetch_all(&db)
            .await?;
        assert!(subtasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 他のユーザーのサブタスク関係は削除できない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;
        let other_user_subtask =
            task_factory::create_subtask(&db, &other_user.id, &other_user_task.id).await?;

        test.login(None).await?;
        let res = test
            .server()
            .delete(&TaskPaths::disconnect_subtask())
            .json(&DisconnectSubtask {
                parent_task_id: other_user_task.id,
                subtask_id: other_user_subtask.id,
            })
            .await;
        res.assert_status_not_ok();

        let subtasks = sqlx::query!("SELECT * FROM subtask_connections;")
            .fetch_all(&db)
            .await?;
        assert!(!subtasks.is_empty());

        Ok(())
    }
}
