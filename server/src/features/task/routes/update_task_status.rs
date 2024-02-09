use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{
            db::{
                find_all_unblocked_descendant_task_ids, is_all_blocking_tasks_done,
                update_task_status, update_tasks_and_ancestors_status, update_tasks_status,
                TaskAndUser, TasksAndUser, UpdateTaskStatusArgs, UpdateTasksStatusArgs,
            },
            UpdateTaskStatus,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::TaskPaths::update_task_status_open_api(), responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskStatus>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    // ブロックしているタスクが完了状態かを確認する。
    if !is_all_blocking_tasks_done(&mut tx, &id).await? {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("ブロックしているタスクが全て完了状態ではありません"),
        ));
    }

    let updated_task = update_task_status(
        &mut tx,
        UpdateTaskStatusArgs {
            id: &id,
            user_id: &user.id,
            status: &payload.status,
        },
    )
    .await?;

    //　ブロッキングタスクにブロックされていない子孫タスクをすべて更新する
    let descendants = find_all_unblocked_descendant_task_ids(
        &mut tx,
        TaskAndUser {
            task_id: &updated_task.id,
            user_id: &user.id,
        },
    )
    .await?;
    update_tasks_status(
        &mut tx,
        UpdateTasksStatusArgs {
            status: &payload.status,
            user_id: &user.id,
            task_ids: &descendants,
        },
    )
    .await?;

    // メインタスクと、すべての祖先タスクを更新する
    update_tasks_and_ancestors_status(
        &mut tx,
        TasksAndUser {
            task_ids: &vec![id.clone()],
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(Json(updated_task))
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::task::{
            db::{find_task, FindTaskArgs},
            routes::TaskPaths,
            test::task_factory,
            Task, TaskStatus, UpdateTaskStatus,
        },
    };

    #[sqlx::test]
    async fn すべての祖先タスクの状態が更新される(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        // t1 --> t12
        // t12 --> t121
        // t12 --> t122
        // t12 --> t123
        let t1 = task_factory::create_with_user(&db, &user.id).await?;
        let t12 = task_factory::create_default_subtask(&db, &user.id, &t1.id).await?;
        let t121 = task_factory::create_default_subtask(&db, &user.id, &t12.id).await?;
        let t122 = task_factory::create_default_subtask(&db, &user.id, &t12.id).await?;
        let t123 = task_factory::create_default_subtask(&db, &user.id, &t12.id).await?;
        assert!([&t1, &t12, &t121, &t122, &t123]
            .iter()
            .all(|s| s.status == TaskStatus::Todo));

        let r1 = test
            .server()
            .put(&TaskPaths::one_update_task_status(&t121.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        r1.assert_status_ok();

        let r2 = test
            .server()
            .put(&TaskPaths::one_update_task_status(&t122.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        r2.assert_status_ok();

        let r3 = test
            .server()
            .put(&TaskPaths::one_update_task_status(&t123.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        r3.assert_status_ok();

        let mut conn = db.acquire().await?;
        let root = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &t1.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(root.status, TaskStatus::Done);

        let mut conn = db.acquire().await?;
        let parent = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &t12.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(parent.status, TaskStatus::Done);

        let mut conn = db.acquire().await?;
        let sub1 = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &t121.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub1.status, TaskStatus::Done);

        let mut conn = db.acquire().await?;
        let sub2 = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &t122.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub2.status, TaskStatus::Done);

        let mut conn = db.acquire().await?;
        let sub3 = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &t123.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub3.status, TaskStatus::Done);

        Ok(())
    }

    #[sqlx::test]
    async fn サブタスクが全て完了にならないと親タスクは完了にならない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        // t1 --> t2
        // t1 --> t3
        let t1 = task_factory::create_with_user(&db, &user.id).await?;
        let t2 = task_factory::create_default_subtask(&db, &user.id, &t1.id).await?;
        let t3 = task_factory::create_default_subtask(&db, &user.id, &t1.id).await?;
        assert!([&t1, &t2, &t3].iter().all(|t| t.status == TaskStatus::Todo));

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&t2.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let parent = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &t1.id,
            },
        )
        .await?;
        assert_eq!(parent.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn 親タスクが完了状態になるとサブタスクもすべて完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        // parent --> sub1
        // parent --> sub2
        // sub1 --> sub11
        let parent = task_factory::create_with_user(&db, &user.id).await?;
        let sub1 = task_factory::create_subtask(
            &db,
            &parent.id,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let sub2 = task_factory::create_subtask(
            &db,
            &parent.id,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let sub11 = task_factory::create_subtask(
            &db,
            &sub1.id,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&parent.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let sub1 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub1.id,
            },
        )
        .await?;
        let sub2 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub2.id,
            },
        )
        .await?;
        let sub11 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub11.id,
            },
        )
        .await?;
        assert_eq!(sub1.status, TaskStatus::Done);
        assert_eq!(sub2.status, TaskStatus::Done);
        assert_eq!(sub11.status, TaskStatus::Done);

        Ok(())
    }

    #[sqlx::test]
    async fn 親タスクが未完了になるとサブタスクもすべて未完了になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        // parent --> sub1
        // parent --> sub2
        // sub1 --> sub11
        let parent = task_factory::create_with_user(&db, &user.id).await?;
        let sub1 = task_factory::create_subtask(
            &db,
            &parent.id,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let sub2 = task_factory::create_subtask(
            &db,
            &parent.id,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let sub11 = task_factory::create_subtask(
            &db,
            &sub1.id,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&parent.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Todo,
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let sub1 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub1.id,
            },
        )
        .await?;
        let sub2 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub2.id,
            },
        )
        .await?;
        let sub11 = find_task(
            &mut conn,
            FindTaskArgs {
                user_id: &user.id,
                task_id: &sub11.id,
            },
        )
        .await?;
        assert_eq!(sub1.status, TaskStatus::Todo);
        assert_eq!(sub2.status, TaskStatus::Todo);
        assert_eq!(sub11.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn ブロックしているタスクが完了している場合は状態を更新できる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let blocked =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        assert_eq!(blocked.status, TaskStatus::Todo);

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&blocked.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let task = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &blocked.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(task.status, TaskStatus::Done);

        Ok(())
    }

    #[sqlx::test]
    async fn ブロックしているタスクが未完了の場合は状態を更新できない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let blocked =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        assert_eq!(blocked.status, TaskStatus::Todo);

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&blocked.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        res.assert_status_not_ok();

        let mut conn = db.acquire().await?;
        let task = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &blocked.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(task.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn 祖先タスクをブロックしているタスクが未完了の場合は状態を更新できない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let main = task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;
        let sub = task_factory::create_default_subtask(&db, &user.id, &main.id).await?;
        assert_eq!(sub.status, TaskStatus::Todo);

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&sub.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
            })
            .await;
        res.assert_status_not_ok();

        let mut conn = db.acquire().await?;
        let task = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &sub.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(task.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn ブロッキングタスクが完了状態ではないサブタスクの状態は更新されない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create_with_user(&db, &user.id).await?;
        assert_eq!(main.status, TaskStatus::Todo);
        let sub = task_factory::create_default_subtask(&db, &user.id, &main.id).await?;
        assert_eq!(sub.status, TaskStatus::Todo);
        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        task_factory::create_blocking_connection(&db, &user.id, &blocking.id, &sub.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&main.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
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

        let sub = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &sub.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn すべてのブロッキングタスクが完了状態ではない場合はサブタスクの状態は更新されない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create_with_user(&db, &user.id).await?;
        assert_eq!(main.status, TaskStatus::Todo);
        let sub = task_factory::create_default_subtask(&db, &user.id, &main.id).await?;
        assert_eq!(sub.status, TaskStatus::Todo);
        let blocking1 = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        task_factory::create_blocking_connection(&db, &user.id, &blocking1.id, &sub.id).await?;
        let blocking2 = task_factory::create_with_user(&db, &user.id).await?;
        task_factory::create_blocking_connection(&db, &user.id, &blocking2.id, &sub.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&main.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
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

        let sub = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &sub.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub.status, TaskStatus::Todo);

        Ok(())
    }

    #[sqlx::test]
    async fn ブロッキングタスクが完了状態のサブタスクは更新される(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let main = task_factory::create_with_user(&db, &user.id).await?;
        assert_eq!(main.status, TaskStatus::Todo);
        let sub = task_factory::create_default_subtask(&db, &user.id, &main.id).await?;
        assert_eq!(sub.status, TaskStatus::Todo);
        let blocking = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Done,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;
        assert_eq!(blocking.status, TaskStatus::Done);
        task_factory::create_blocking_connection(&db, &user.id, &blocking.id, &sub.id).await?;

        let res = test
            .server()
            .put(&TaskPaths::one_update_task_status(&main.id))
            .json(&UpdateTaskStatus {
                status: TaskStatus::Done,
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

        let sub = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &sub.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(sub.status, TaskStatus::Done);

        Ok(())
    }
}
