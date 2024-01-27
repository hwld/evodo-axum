use axum::{extract::State, Json};
use axum_garde::WithValidation;
use http::StatusCode;

use crate::{
    features::task::{CreateTask, Task},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::TASKS_PATH, request_body = CreateTask, responses((status = 201, body = Task)))]
pub async fn handler(
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateTask>>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING *"#,
        uuid,
        payload.title,
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

#[cfg(test)]
mod tests {
    use crate::{
        app::tests,
        features::task::{routers::TASKS_PATH, CreateTask, Task},
        AppResult, Db,
    };

    #[sqlx::test]
    async fn タスクを作成できる(db: Db) -> AppResult<()> {
        let title = "title";

        let server = tests::build(db.clone()).await?;
        let res_task: Task = server
            .post(TASKS_PATH)
            .json(&CreateTask {
                title: title.into(),
            })
            .await
            .json();

        let created = sqlx::query_as!(Task, "SELECT * FROM tasks where id = $1", res_task.id)
            .fetch_all(&db)
            .await?;
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, title);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字のタスクを作成できない(db: Db) -> AppResult<()> {
        let server = tests::build(db.clone()).await?;
        server
            .post(TASKS_PATH)
            .json(&CreateTask { title: "".into() })
            .await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 0);
        Ok(())
    }
}
