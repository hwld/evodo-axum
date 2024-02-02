use axum::{extract::State, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::features::task::db::{insert_task, InsertTaskArgs};
use crate::{
    app::AppState,
    error::AppError,
    features::{auth::Auth, task::CreateTask},
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::TaskPaths::tasks(), request_body = CreateTask, responses((status = 201, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateTask>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let uuid = uuid::Uuid::new_v4().to_string();
    let task = insert_task(
        &mut tx,
        InsertTaskArgs {
            id: &uuid,
            title: &payload.title,
            user_id: &user.id,
            status: &Default::default(),
        },
    )
    .await?;

    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(task)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::app::AppResult;
    use crate::features::task::db::{find_task, FindTaskArgs};
    use crate::{
        app::{tests::AppTest, Db},
        features::task::{routes::TaskPaths, CreateTask, Task},
    };

    #[sqlx::test]
    async fn タスクを作成できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let title = "title";
        let task: Task = test
            .server()
            .post(&TaskPaths::tasks())
            .json(&CreateTask {
                title: title.into(),
            })
            .await
            .json();

        let mut conn = db.acquire().await?;
        let created = find_task(
            &mut conn,
            FindTaskArgs {
                task_id: &task.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(created.title, title);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字のタスクを作成できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        test.login(None).await?;

        let res = test
            .server()
            .post(&TaskPaths::tasks())
            .json(&CreateTask { title: "".into() })
            .await;
        res.assert_status_not_ok();

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert!(tasks.is_empty());
        Ok(())
    }
}
