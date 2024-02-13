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
            db::SubTaskConnectionError,
            usecases::reconnect_sub_task::{self, ReconnectSubTaskArgs, ReconnectSubTaskError},
            ReconnectSubTask,
        },
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub enum ReconnectSubTaskErrorType {
    TaskNotFound,
    BlockedByMainTask,
    CircularTask,
    MultipleMainTask,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReconnectSubTaskErrorBody {
    error_type: ReconnectSubTaskErrorType,
}

#[tracing::instrument(err)]
#[utoipa::path(
    put,
    tag = super::TAG,
    path = super::TaskPaths::reconnect_sub_task(),
    responses(
        (status = 200),
        (status = 400, body = ReconnectSubTaskErrorBody)
    )
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ReconnectSubTask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let result = reconnect_sub_task::action(
        &mut tx,
        ReconnectSubTaskArgs {
            old_main_task_id: &payload.old_main_task_id,
            old_sub_task_id: &payload.old_sub_task_id,
            new_main_task_id: &payload.new_main_task_id,
            new_sub_task_id: &payload.new_sub_task_id,
            user_id: &user.id,
        },
    )
    .await;
    if let Err(e) = result {
        use ReconnectSubTaskError::{Connect, Unknown};
        use ReconnectSubTaskErrorType::{
            BlockedByMainTask, CircularTask, MultipleMainTask, TaskNotFound,
        };

        let error_type = match e {
            Connect(SubTaskConnectionError::TaskNotFound) => TaskNotFound,
            Connect(SubTaskConnectionError::BlockedByMainTask) => BlockedByMainTask,
            Connect(SubTaskConnectionError::CircularTask) => CircularTask,
            Connect(SubTaskConnectionError::MultipleMainTask) => MultipleMainTask,
            Connect(SubTaskConnectionError::Unknown(_)) | Unknown(_) => {
                return Err(anyhow!("Unknown").into())
            }
        };

        return Err(AppError::with_json(
            StatusCode::BAD_REQUEST,
            ReconnectSubTaskErrorBody { error_type },
        ));
    }

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            task::{routes::TaskPaths, test::task_factory, ReconnectSubTask},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn サブタスクの再接続ができる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 = task_factory::create_default_sub_task(&db, &user.id, &task1.id.clone()).await?;
        let task3 = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::reconnect_sub_task())
            .json(&ReconnectSubTask {
                old_main_task_id: task1.id.clone(),
                old_sub_task_id: task2.id.clone(),
                new_main_task_id: task2.id.clone(),
                new_sub_task_id: task3.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let deleted = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2;",
            task1.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(deleted.is_empty());

        let created = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
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
        let task2 = task_factory::create_default_sub_task(&db, &user.id, &task1.id).await?;
        let task3 = task_factory::create_default_sub_task(&db, &user.id, &task2.id).await?;
        let task4 = task_factory::create_default_sub_task(&db, &user.id, &task3.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_sub_task())
            .json(&ReconnectSubTask {
                old_main_task_id: task3.id.clone(),
                old_sub_task_id: task4.id.clone(),
                new_main_task_id: task3.id.clone(),
                new_sub_task_id: task2.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let old = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id= $1 AND sub_task_id = $2",
            task3.id,
            task4.id
        )
        .fetch_all(&db)
        .await?;
        assert_eq!(old.len(), 1);

        let sub_tasks = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
            task3.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(sub_tasks.is_empty());

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
            task_factory::create_default_sub_task(&db, &other_user.id, &other_task1.id).await?;

        let user = test.login(None).await?;
        let my_task = task_factory::create_with_user(&db, &user.id).await?;
        let res = test
            .server()
            .put(&TaskPaths::reconnect_sub_task())
            .json(&ReconnectSubTask {
                old_main_task_id: other_task1.id.clone(),
                old_sub_task_id: other_task2.id.clone(),
                new_main_task_id: other_task2.id.clone(),
                new_sub_task_id: my_task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let old_connection = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2 AND user_id = $3",
            other_task1.id,
            other_task2.id,
            other_user.id
        )
        .fetch_all(&db)
        .await?;
        assert_eq!(old_connection.len(), 1);

        let new_connection = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2 AND user_id = $3",
            other_task2.id,
            my_task.id,
            user.id
        )
        .fetch_all(&db)
        .await?;
        assert!(new_connection.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 自分自身をサブタスクにはできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create_with_user(&db, &user.id).await?;
        let sub_task = task_factory::create_default_sub_task(&db, &user.id, &task.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::reconnect_sub_task())
            .json(&ReconnectSubTask {
                old_main_task_id: task.id.clone(),
                old_sub_task_id: sub_task.id.clone(),
                new_main_task_id: task.id.clone(),
                new_sub_task_id: task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let new_sub_task = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $1;",
            task.id
        )
        .fetch_optional(&db)
        .await?;
        assert!(new_sub_task.is_none());

        Ok(())
    }
}
