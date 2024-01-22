use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{
    features::{
        task::Task,
        task_node::{TaskNode, TaskNodeInfo},
    },
    AppError,
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = "task-node", path = "/task-nodes", responses((status = 200, body = [TaskNode])))]
pub async fn handler(
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Vec<TaskNode>>), AppError> {
    let records = sqlx::query!(
        // https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        // ここを見ると、MySQLの場合はONでnot nullのフィールドを比較してたらnon-nullになるっぽいけど、
        // sqliteとpostgresqlではならなそうなので "field!"で上書きする
        r#"
        SELECT
            n.*,
            t.status as "status!",
            t.title as "title!",
            t.created_at as "created_at!",
            t.updated_at as "updated_at!"
        FROM 
            task_node_info as n LEFT JOIN tasks as t
                ON n.task_id = t.id
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let nodes: Vec<TaskNode> = records
        .into_iter()
        .map(|r| TaskNode {
            task: Task {
                id: r.task_id.clone(),
                title: r.title,
                status: r.status.into(),
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            node_info: TaskNodeInfo {
                id: r.id,
                task_id: r.task_id,
                x: r.x,
                y: r.y,
            },
        })
        .collect();

    Ok((StatusCode::OK, Json(nodes)))
}
