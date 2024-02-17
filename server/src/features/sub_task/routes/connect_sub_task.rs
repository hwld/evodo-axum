use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        sub_task::{
            usecases::connect_sub_task::{self, ConnectSubTaskArgs, ConnectSubTaskError},
            ConnectSubTask,
        },
        task::db::SubTaskConnectionError,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub enum ConnectSubTaskErrorType {
    TaskNotFound,
    CircularTask,
    MultipleMainTask,
    BlockedByMainTask,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectSubTaskErrorBody {
    error_type: ConnectSubTaskErrorType,
}

#[tracing::instrument(err)]
#[utoipa::path(
    post,
    tag = super::TAG,
    path = super::SubTaskPaths::connect_sub_task(),
    responses(
        (status = 200),
        (status = 400, body = ConnectSubTaskErrorBody)
    )
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ConnectSubTask>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    if let Err(e) = connect_sub_task::action(
        &mut tx,
        ConnectSubTaskArgs {
            main_task_id: &payload.main_task_id,
            sub_task_id: &payload.sub_task_id,
            user_id: &user.id,
        },
    )
    .await
    {
        use ConnectSubTaskError::{CheckError, Unknown};
        use ConnectSubTaskErrorType::{
            BlockedByMainTask, CircularTask, MultipleMainTask, TaskNotFound,
        };

        let error_type = match e {
            CheckError(SubTaskConnectionError::TaskNotFound) => TaskNotFound,
            CheckError(SubTaskConnectionError::CircularTask) => CircularTask,
            CheckError(SubTaskConnectionError::MultipleMainTask) => MultipleMainTask,
            CheckError(SubTaskConnectionError::BlockedByMainTask) => BlockedByMainTask,
            CheckError(SubTaskConnectionError::Unknown(_)) | Unknown(_) => {
                return Err(anyhow!("Unknown").into());
            }
        };

        return Err(AppError::with_json(
            StatusCode::BAD_REQUEST,
            ConnectSubTaskErrorBody { error_type },
        ));
    };

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::app::{tests::AppTest, AppResult, Db};
    use crate::features::sub_task::routes::SubTaskPaths;
    use crate::features::sub_task::ConnectSubTask;
    use crate::features::task::db::{find_task, FindTaskArgs};
    use crate::features::task::test::task_factory::{self};
    use crate::features::task::{Task, TaskStatus};
    use crate::features::user::test::user_factory;

    #[sqlx::test]
    async fn サブタスクを作成できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main_task = task_factory::create_with_user(&db, &user.id).await?;
        let sub_task = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: main_task.id.clone(),
                sub_task_id: sub_task.id.clone(),
            })
            .await;

        let fetched_sub_task = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1;",
            main_task.id
        )
        .fetch_one(&db)
        .await?;
        res.assert_status_ok();

        assert_eq!(sub_task.id, fetched_sub_task.sub_task_id);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のユーザーのタスクを指定できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task1 = task_factory::create_with_user(&db, &other_user.id).await?;
        let other_user_task2 = task_factory::create_with_user(&db, &other_user.id).await?;

        test.login(None).await?;
        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: other_user_task1.id,
                sub_task_id: other_user_task2.id,
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!("SELECT * FROM sub_tasks;")
            .fetch_all(&db)
            .await?;

        assert!(sub_tasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn サブタスク関係を相互に持たせることはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task2 = task_factory::create_default_sub_task(&db, &user.id, &task1.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: task2.id.clone(),
                sub_task_id: task1.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2;",
            task2.id,
            task1.id
        )
        .fetch_all(&db)
        .await?;
        assert!(sub_tasks.is_empty());

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
        let task5 = task_factory::create_default_sub_task(&db, &user.id, &task4.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: task5.id.clone(),
                sub_task_id: task2.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
            task5.id,
            task2.id
        )
        .fetch_all(&db)
        .await?;
        assert!(sub_tasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 自分自身をサブタスクにはできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create_with_user(&db, &user.id).await?;
        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: task.id.clone(),
                sub_task_id: task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!("SELECT * FROM sub_tasks;")
            .fetch_all(&db)
            .await?;
        assert_eq!(sub_tasks.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn 完了状態のタスクをサブタスクにすると完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create_with_user(&db, &user.id).await?;
        let sub = task_factory::create(
            &db,
            Task {
                user_id: user.id.clone(),
                status: TaskStatus::Done,
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: main.id.clone(),
                sub_task_id: sub.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let main = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &main.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(main.status, TaskStatus::Done);

        Ok(())
    }

    #[sqlx::test]
    async fn 未完了状態のタスクをサブタスクにすると未完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let _done_sub = task_factory::create_sub_task(
            &db,
            &main.id,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let todo_sub = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: main.id.clone(),
                sub_task_id: todo_sub.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let main = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &main.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(main.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    pub fn メインタスクをブロックしているタスクをサブタスクにはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let main = task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        let sub = task_factory::create_default_sub_task(&db, &user.id, &main.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: sub.id.clone(),
                sub_task_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
            sub.id,
            blocking.id
        )
        .fetch_all(&db)
        .await?;
        assert!(sub_tasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    pub fn ブロックしているタスクをサブタスクにすることはできない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: blocking.id.clone(),
                sub_task_id: blocked.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let res = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
            blocking.id,
            blocked.id
        )
        .fetch_all(&db)
        .await?;
        assert!(res.is_empty());

        Ok(())
    }

    #[sqlx::test]
    pub fn 複数のタスクのサブタスクにはなれない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main1 = task_factory::create_with_user(&db, &user.id).await?;
        let main2 = task_factory::create_with_user(&db, &user.id).await?;
        let sub = task_factory::create_default_sub_task(&db, &user.id, &main1.id).await?;

        let res = test
            .server()
            .post(&SubTaskPaths::connect_sub_task())
            .json(&ConnectSubTask {
                main_task_id: main2.id.clone(),
                sub_task_id: sub.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let res = sqlx::query!(
            "SELECT * FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2",
            main2.id,
            sub.id
        )
        .fetch_all(&db)
        .await?;
        assert!(res.is_empty());

        Ok(())
    }
}
