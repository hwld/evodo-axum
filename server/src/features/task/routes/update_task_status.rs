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
                find_task, update_all_ancestors_task_status, update_task_status, FindTaskArgs,
                TaskAndUser, UpdateTaskStatusArgs,
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

    let task = find_task(
        &mut tx,
        FindTaskArgs {
            task_id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    // TODO: Doneに変更するのは子孫をすべてDoneにすれば良いので実装できそう
    // サブタスクを持っているタスクは直接更新できない
    if !task.subtask_ids.is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("サブタスクを持っているタスクは直接更新できない"),
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

    // すべての祖先タスクを更新する
    update_all_ancestors_task_status(
        &mut tx,
        TaskAndUser {
            task_id: &updated_task.id,
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
            TaskStatus, UpdateTaskStatus,
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
        let t12 = task_factory::create_subtask(&db, &user.id, &t1.id).await?;
        let t121 = task_factory::create_subtask(&db, &user.id, &t12.id).await?;
        let t122 = task_factory::create_subtask(&db, &user.id, &t12.id).await?;
        let t123 = task_factory::create_subtask(&db, &user.id, &t12.id).await?;
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
        let t2 = task_factory::create_subtask(&db, &user.id, &t1.id).await?;
        let t3 = task_factory::create_subtask(&db, &user.id, &t1.id).await?;
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
}
