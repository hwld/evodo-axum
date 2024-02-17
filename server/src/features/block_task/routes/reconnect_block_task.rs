use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_login::AuthSession;
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        block_task::{
            usecases::reconnect_block_task::{
                self, ReconnectBlockTaskArgs, ReconnectBlockTaskError,
            },
            ReconnectBlockTask,
        },
        task::db::BlockTaskConnectionError,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub enum ReconnectBlockTaskErrorType {
    TaskNotFound,
    IsSubTask,
    CircularTask,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReconnectBlockTaskErrorBody {
    error_type: ReconnectBlockTaskErrorType,
}

#[tracing::instrument(err)]
#[utoipa::path(
    put,
    tag = super::TAG,
    path = super::BlockTaskPaths::reconnect_block_task(),
    responses(
        (status = 200),
        (status = 400, body = ReconnectBlockTaskErrorBody)
    )
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ReconnectBlockTask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let result = reconnect_block_task::action(
        &mut tx,
        ReconnectBlockTaskArgs {
            old_blocking_task_id: &payload.old_blocking_task_id,
            old_blocked_task_id: &payload.old_blocked_task_id,
            new_blocking_task_id: &payload.new_blocking_task_id,
            new_blocked_task_id: &payload.new_blocked_task_id,
            user_id: &user.id,
        },
    )
    .await;
    if let Err(e) = result {
        use ReconnectBlockTaskError::{Connect, Unknown};
        use ReconnectBlockTaskErrorType::{CircularTask, IsSubTask, TaskNotFound};

        let error_type = match e {
            Connect(BlockTaskConnectionError::TaskNotFound) => TaskNotFound,
            Connect(BlockTaskConnectionError::CircularTask) => CircularTask,
            Connect(BlockTaskConnectionError::IsSubTask) => IsSubTask,
            Connect(BlockTaskConnectionError::Unknown(_)) | Unknown(_) => {
                return Err(anyhow!("Unknown").into());
            }
        };

        return Err(AppError::with_json(
            StatusCode::BAD_REQUEST,
            ReconnectBlockTaskErrorBody { error_type },
        ));
    };

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            block_task::{routes::BlockTaskPaths, ReconnectBlockTask},
            task::test::task_factory,
        },
    };

    #[sqlx::test]
    async fn ブロックタスクの再接続ができる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 =
            task_factory::create_default_blocked_task(&db, &user.id, &task1.id.clone()).await?;
        let task3 = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .put(&BlockTaskPaths::reconnect_block_task())
            .json(&ReconnectBlockTask {
                old_blocking_task_id: task1.id.clone(),
                old_blocked_task_id: task2.id.clone(),
                new_blocking_task_id: task2.id.clone(),
                new_blocked_task_id: task3.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let deleted = sqlx::query!(
            "SELECT * FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2;",
            task1.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(deleted.is_empty());

        let created = sqlx::query!(
            "SELECT * FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2",
            task2.id,
            task3.id
        )
        .fetch_all(&db)
        .await?;
        assert!(!created.is_empty());

        Ok(())
    }
}
