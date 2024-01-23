use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::Acquire;
use sqlx::{Pool, Sqlite};

use crate::features::task::Task;
use crate::features::task_node::{CreateTaskNode, TaskNode, TaskNodeInfo};
use crate::AppResult;

#[tracing::instrument(err)]
#[utoipa::path(post, tag = "task-node", path = "/task-nodes", responses((status = 201, body = TaskNode)))]
pub async fn handler(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTaskNode>,
) -> AppResult<(StatusCode, Json<TaskNode>)> {
    let mut tx = pool.begin().await?;

    let task_id = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING * "#,
        task_id,
        payload.task.title
    )
    .fetch_one(tx.acquire().await?)
    .await?;

    let node_info_id = uuid::Uuid::new_v4().to_string();
    let node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#" INSERT INTO task_node_info(id, task_id, x, y) VALUES($1, $2, $3, $4) RETURNING * "#,
        node_info_id,
        task.id,
        payload.x,
        payload.y
    )
    .fetch_one(tx.acquire().await?)
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(TaskNode { task, node_info })))
}
