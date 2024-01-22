use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{features::task::Task, AppError};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = "task", path = "/tasks/{id}", responses((status = 200, body = Task)))]
pub async fn handler(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let task = sqlx::query_as!(Task, r#"DELETE FROM tasks WHERE id = $1 RETURNING *"#, id)
        .fetch_one(&pool)
        .await?;

    Ok((StatusCode::OK, Json(task)))
}
