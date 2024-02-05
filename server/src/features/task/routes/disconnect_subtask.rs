use axum::{extract::State, Json};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{
            usecases::disconnect_subtask::{self, DisconnectSubtaskArgs},
            DisconnectSubtask,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::TaskPaths::disconnect_subtask(), responses(( status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<DisconnectSubtask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    disconnect_subtask::action(
        &mut tx,
        DisconnectSubtaskArgs {
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
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            task::{
                db::{find_task, FindTaskArgs},
                routes::TaskPaths,
                test::task_factory,
                DisconnectSubtask, Task, TaskStatus,
            },
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn サブタスク関係を削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create_with_user(&db, &user.id).await?;
        let subtask = task_factory::create_default_subtask(&db, &user.id, &task.id).await?;

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
            task_factory::create_default_subtask(&db, &other_user.id, &other_user_task.id).await?;

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

    #[sqlx::test]
    async fn 完了していないサブタスク関係を削除すると親は完了状態になる(
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
            .delete(&TaskPaths::disconnect_subtask())
            .json(&DisconnectSubtask {
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
        assert_eq!(parent.status, TaskStatus::Done);

        Ok(())
    }
}
