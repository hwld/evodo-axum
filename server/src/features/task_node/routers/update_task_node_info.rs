use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{
    features::task_node::{TaskNodeInfo, UpdateTaskNodeInfo},
    AppResult,
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = "task-node", path = "/task-node-info/{id}", responses((status = 200, body = TaskNodeInfo)))]
pub async fn handler(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateTaskNodeInfo>,
) -> AppResult<(StatusCode, Json<TaskNodeInfo>)> {
    let task_node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#"
        UPDATE
            task_node_info
        SET
            x = $1,
            y = $2
        WHERE
            id = $3
        RETURNING *;
        "#,
        payload.x,
        payload.y,
        id,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::OK, Json(task_node_info)))
}
