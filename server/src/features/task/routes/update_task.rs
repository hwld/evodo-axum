use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::{
    app::AppState,
    error::AppError,
    features::{
        auth::Auth,
        task::{Task, UpdateTask},
    },
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::Paths::oas_task(), request_body = UpdateTask, responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<UpdateTask>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, None));
    };

    let task = sqlx::query_as!(
        Task,
        r#"
            UPDATE
                tasks 
            SET
                status = $1,
                title = $2,
                updated_at = (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime'))
            WHERE
                id = $3 AND user_id = $4
            RETURNING *;
        "#,
        payload.status,
        payload.title,
        id,
        user.id
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task)).into_response())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::AppResult;
    use crate::{
        app::{tests::AppTest, Db},
        features::{
            task::{routes::Paths, test::factory as task_factory, TaskStatus},
            user::test::factory as user_factory,
        },
    };

    #[sqlx::test]
    async fn タスクを更新できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task = task_factory::create(
            &db,
            Task {
                user_id: user.id,
                title: "old".into(),
                status: TaskStatus::Todo,
                ..Default::default()
            },
        )
        .await?;
        let new_title = "new_title";
        let new_status = TaskStatus::Done;

        test.server()
            .put(&Paths::one_task(&task.id))
            .json(&UpdateTask {
                title: new_title.into(),
                status: new_status,
            })
            .await;

        let updated = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task.id)
            .fetch_one(&db)
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
                user_id: user.id,
                title: old_title.into(),
                ..Default::default()
            },
        )
        .await?;

        let res = test
            .server()
            .put(&Paths::one_task(&old_task.id))
            .json(&UpdateTask {
                title: "".into(),
                status: TaskStatus::Todo,
            })
            .await;
        res.assert_status_not_ok();

        let task = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", old_task.id)
            .fetch_one(&db)
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
                user_id: other_user.id,
                ..Default::default()
            },
        )
        .await?;

        let new_title = "new_title";
        let new_status = TaskStatus::Done;

        test.login(None).await?;
        test.server()
            .post(&Paths::one_task(&other_user_task.id))
            .json(&UpdateTask {
                title: new_title.into(),
                status: new_status,
            })
            .await;

        let task = sqlx::query_as!(
            Task,
            "SELECT * FROM tasks WHERE id = $1",
            other_user_task.id
        )
        .fetch_one(&db)
        .await?;
        assert_eq!(task.title, other_user_task.title);
        assert_eq!(task.status, other_user_task.status);

        Ok(())
    }
}