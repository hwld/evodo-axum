use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::Acquire;

use crate::features::task::Task;
use crate::features::task_node::{CreateTaskNode, TaskNode, TaskNodeInfo};
use crate::{AppResult, Db};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = "task-node", path = "/task-nodes", responses((status = 201, body = TaskNode)))]
pub async fn handler(
    State(db): State<Db>,
    Json(payload): Json<CreateTaskNode>,
) -> AppResult<(StatusCode, Json<TaskNode>)> {
    let mut tx = db.begin().await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{features::task::CreateTask, AppResult};

    #[sqlx::test]
    async fn タスクノードを作成できる(db: Db) -> AppResult<()> {
        let task_title = "title";
        let node_x = 0.0;
        let node_y = 0.0;

        let (_, task_node) = handler(
            State(db.clone()),
            Json(CreateTaskNode {
                x: node_x,
                y: node_y,
                task: CreateTask {
                    title: task_title.into(),
                },
            }),
        )
        .await?;

        let task = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task_node.task.id)
            .fetch_one(&db)
            .await?;
        assert_eq!(task.title, task_title);

        let node_info = sqlx::query_as!(
            TaskNodeInfo,
            "SELECT * FROM task_node_info WHERE id = $1",
            task_node.node_info.id
        )
        .fetch_one(&db)
        .await?;
        assert_eq!(node_info.x, node_x);
        assert_eq!(node_info.y, node_y);

        Ok(())
    }
}
