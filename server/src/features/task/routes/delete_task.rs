use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::AppResult,
    features::task::{
        db::{
            delete_task, find_parent_task_ids, update_tasks_and_ancestors_status, DeleteTaskArgs,
            FindParentTaskIdsArgs, TasksAndUser,
        },
        DeleteTaskResponse,
    },
};
use crate::{app::AppState, error::AppError, features::auth::Auth};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::TaskPaths::task_open_api(), responses((status = 200, body = DeleteTaskResponse)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    // 祖先タスクを更新するために削除する前に取得しておく
    let parent_ids = find_parent_task_ids(
        &mut tx,
        FindParentTaskIdsArgs {
            subtask_id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    let deleted_id = delete_task(
        &mut tx,
        DeleteTaskArgs {
            id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    // 祖先タスクを更新
    update_tasks_and_ancestors_status(
        &mut tx,
        TasksAndUser {
            task_ids: &parent_ids,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(DeleteTaskResponse {
            task_id: deleted_id,
        }),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use crate::app::Db;
    use crate::features::task::db::{find_task, FindTaskArgs};
    use crate::features::task::test::task_factory;
    use crate::features::task::{Task, TaskStatus};
    use crate::features::user::test::user_factory;
    use crate::{app::tests::AppTest, features::task::routes::TaskPaths};

    use super::*;

    #[sqlx::test]
    async fn 指定したタスクだけを削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        task_factory::create_with_user(&db, &user.id).await?;
        let created_task = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .delete(&TaskPaths::one_task(&created_task.id))
            .await;
        res.assert_status_ok();

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクは削除できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;

        test.login(None).await?;
        let res = test
            .server()
            .delete(&TaskPaths::one_task(&other_user_task.id))
            .await;
        assert_ne!(res.status_code(), StatusCode::UNAUTHORIZED);

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn 完了していないサブタスクを削除するとメインタスクが完了状態になる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let parent = task_factory::create(
            &db,
            Task {
                status: TaskStatus::Todo,
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
        let todo_sub = task_factory::create_subtask(
            &db,
            &parent.id,
            Task {
                status: TaskStatus::Todo,
                user_id: user.id.clone(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .delete(&TaskPaths::one_task(&todo_sub.id))
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
}
