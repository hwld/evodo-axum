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
        task::{
            db::BlockTaskConnectionError,
            usecases::connect_block_task::{self, ConnectBlockTaskArgs, ConnectBlockTaskError},
            ConnectBlockTask,
        },
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub enum ConnectBlockTaskErrorType {
    TaskNotFound,
    IsSubTask,
    CircularTask,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectBlockTaskErrorBody {
    error_type: ConnectBlockTaskErrorType,
}

#[tracing::instrument(err)]
#[utoipa::path(
    post,
    tag = super::TAG,
    path = super::TaskPaths::connect_block_task(),
    responses(
        (status = 200),
        (status = 400, body = ConnectBlockTaskErrorBody)
    )
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ConnectBlockTask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    if let Err(e) = connect_block_task::action(
        &mut tx,
        ConnectBlockTaskArgs {
            blocking_task_id: &payload.blocking_task_id,
            blocked_task_id: &payload.blocked_task_id,
            user_id: &user.id,
        },
    )
    .await
    {
        use ConnectBlockTaskError::{CheckError, Unknown};
        use ConnectBlockTaskErrorType::{CircularTask, IsSubTask, TaskNotFound};

        let error_type = match e {
            CheckError(BlockTaskConnectionError::TaskNotFound) => TaskNotFound,
            CheckError(BlockTaskConnectionError::IsSubTask) => IsSubTask,
            CheckError(BlockTaskConnectionError::CircularTask) => CircularTask,
            CheckError(BlockTaskConnectionError::Unknown(_)) | Unknown(_) => {
                return Err(anyhow!("Unknown").into())
            }
        };

        return Err(AppError::with_json(
            StatusCode::BAD_REQUEST,
            ConnectBlockTaskErrorBody { error_type },
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
            task::{routes::TaskPaths, test::task_factory, ConnectBlockTask},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn タスクをブロックタスクにできる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: blocked.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_one(&db)
            .await?;
        assert_eq!(block.blocking_task_id, blocking.id);
        assert_eq!(block.blocked_task_id, blocked.id);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクをブロックタスクにできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: other_user_task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(block.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 自分自身をブロックできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(block.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn メインタスクはサブタスクをブロックできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create_with_user(&db, &user.id).await?;
        let sub1 = task_factory::create_default_sub_task(&db, &user.id, &main.id).await?;
        let sub11 = task_factory::create_default_sub_task(&db, &user.id, &sub1.id).await?;
        let _sub12 = task_factory::create_default_sub_task(&db, &user.id, &sub1.id).await?;
        let _sub111 = task_factory::create_default_sub_task(&db, &user.id, &sub11.id).await?;
        let sub112 = task_factory::create_default_sub_task(&db, &user.id, &sub11.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: main.id.clone(),
                blocked_task_id: sub112.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let blocks = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(blocks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn ブロックタスクを循環させることはできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked1 =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        let blocked11 =
            task_factory::create_default_blocked_task(&db, &user.id, &blocked1.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocked11.id.clone(),
                blocked_task_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let result = sqlx::query!(
            "SELECT * FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2;",
            blocked11.id,
            blocking.id
        )
        .fetch_all(&db)
        .await?;
        assert!(result.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn メインタスクをブロックしているタスクをブロックすることはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        // blocking -->|block| blocked1
        // blocked1 -->|sub| sub
        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked1 =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        let sub = task_factory::create_default_sub_task(&db, &user.id, &blocked1.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: sub.id.clone(),
                blocked_task_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let result = sqlx::query!(
            "SELECT * FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2;",
            sub.id,
            blocking.id
        )
        .fetch_all(&db)
        .await?;
        assert!(result.is_empty());

        Ok(())
    }
}
