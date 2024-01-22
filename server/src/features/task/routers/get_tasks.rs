use anyhow::Result;
use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{features::task::Task, AppError};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = "task", path = "/tasks", responses((status = 200, body = [Task])))]
pub async fn handler(
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Vec<Task>>), AppError> {
    let tasks = sqlx::query_as!(Task, r#"select * from tasks;"#)
        .fetch_all(&pool)
        .await?;

    Ok((StatusCode::OK, Json(tasks)))
}
