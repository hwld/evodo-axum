use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::AppResult,
    features::task::db::{update_task, UpdateTaskArgs},
};
use crate::{
    app::AppState,
    error::AppError,
    features::{auth::Auth, task::UpdateTask},
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::TaskPaths::task_open_api(), request_body = UpdateTask, responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<UpdateTask>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let task = update_task(
        &mut tx,
        UpdateTaskArgs {
            id: &id,
            title: &payload.title,
            status: &payload.status,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(task)).into_response())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::AppResult;
    use crate::features::task::db::{find_task, FindTaskArgs};
    use crate::features::task::Task;
    use crate::{
        app::{tests::AppTest, Db},
        features::{
            task::{routes::TaskPaths, test::task_factory, TaskStatus},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    // TODO: DbをConnectionにできないかを考える
    async fn タスクを更新できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create(
            &db,
            Task {
                user_id: user.id.clone(),
                title: "old".into(),
                status: TaskStatus::Todo,
                ..Default::default()
            },
        )
        .await?;
        let new_title = "new_title";
        let new_status = TaskStatus::Done;

        let res = test
            .server()
            .put(&TaskPaths::one_task(&task.id))
            .json(&UpdateTask {
                title: new_title.into(),
                status: new_status,
            })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let updated = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &task.id,
                user_id: &user.id,
            },
        )
        .await?;

        assert_eq!(updated.title, new_title);
        assert_eq!(updated.status, new_status);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字列には更新できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let old_title = "old_title";
        let old_task = task_factory::create(
            &db,
            Task {
                user_id: user.id.clone(),
                title: old_title.into(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .put(&TaskPaths::one_task(&old_task.id))
            .json(&UpdateTask {
                title: "".into(),
                status: TaskStatus::Todo,
            })
            .await;
        res.assert_status_not_ok();

        let mut conn = db.acquire().await?;
        let task = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &old_task.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(task.title, old_title);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクを更新できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create(
            &db,
            Task {
                title: "old_title".into(),
                status: TaskStatus::Todo,
                user_id: other_user.id.clone(),
                ..Default::default()
            },
        )
        .await?;

        let new_title = "new_title";
        let new_status = TaskStatus::Done;

        test.login(None).await?;
        let res = test
            .server()
            .post(&TaskPaths::one_task(&other_user_task.id))
            .json(&UpdateTask {
                title: new_title.into(),
                status: new_status,
            })
            .await;
        res.assert_status_not_ok();

        let mut conn = db.acquire().await?;
        let task = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &other_user_task.id,
                user_id: &other_user.id,
            },
        )
        .await?;
        assert_eq!(task.title, other_user_task.title);
        assert_eq!(task.status, other_user_task.status);

        Ok(())
    }
}
