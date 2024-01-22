use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{
    features::task::{CreateTask, Task},
    AppError,
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = "task", path = "/tasks", responses((status = 201, body = Task)))]
pub async fn handler(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING *"#,
        uuid,
        payload.title,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}
