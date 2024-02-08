use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{
            usecases::connect_subtask::{self, ConnectSubtaskArgs},
            ConnectSubtask,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::TaskPaths::connect_subtask(), responses((status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ConnectSubtask>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    connect_subtask::action(
        &mut tx,
        ConnectSubtaskArgs {
            parent_task_id: &payload.parent_task_id,
            subtask_id: &payload.subtask_id,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::app::{tests::AppTest, AppResult, Db};
    use crate::features::task::db::{find_task, FindTaskArgs};
    use crate::features::task::routes::TaskPaths;
    use crate::features::task::test::task_factory::{self};
    use crate::features::task::{ConnectSubtask, Task, TaskStatus};
    use crate::features::user::test::user_factory;

    #[sqlx::test]
    async fn サブタスクを作成できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent_task = task_factory::create_with_user(&db, &user.id).await?;
        let subtask = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
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
        res.assert_status_ok();

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
        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: other_user_task1.id,
                subtask_id: other_user_task2.id,
            })
            .await;
        res.assert_status_not_ok();

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
        let task2 = task_factory::create_default_subtask(&db, &user.id, &task1.id).await?;

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
        let task2 = task_factory::create_default_subtask(&db, &user.id, &task1.id).await?;
        let task3 = task_factory::create_default_subtask(&db, &user.id, &task2.id).await?;
        let task4 = task_factory::create_default_subtask(&db, &user.id, &task3.id).await?;
        let task5 = task_factory::create_default_subtask(&db, &user.id, &task4.id).await?;

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

    #[sqlx::test]
    async fn 自分自身をサブタスクにはできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create_with_user(&db, &user.id).await?;
        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: task.id.clone(),
                subtask_id: task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let subtasks = sqlx::query!("SELECT * FROM subtask_connections;")
            .fetch_all(&db)
            .await?;
        assert_eq!(subtasks.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn 完了状態のタスクをサブタスクにすると完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent = task_factory::create_with_user(&db, &user.id).await?;
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
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: parent.id.clone(),
                subtask_id: sub.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let parent = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &parent.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(parent.status, TaskStatus::Done);

        Ok(())
    }

    #[sqlx::test]
    async fn 未完了状態のタスクをサブタスクにすると未完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let _done_sub = task_factory::create_subtask(
            &db,
            &parent.id,
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
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: parent.id.clone(),
                subtask_id: todo_sub.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let parent = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &parent.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(parent.status, TaskStatus::Todo);

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
        let sub = task_factory::create_default_subtask(&db, &user.id, &main.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_subtask())
            .json(&ConnectSubtask {
                parent_task_id: sub.id.clone(),
                subtask_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let subtasks = sqlx::query!(
            "SELECT * FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2",
            sub.id,
            blocking.id
        )
        .fetch_all(&db)
        .await?;
        assert!(subtasks.is_empty());

        Ok(())
    }
}
