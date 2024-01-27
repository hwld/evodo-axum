use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;

use crate::{
    features::task_node::{TaskNodeInfo, UpdateTaskNodeInfo},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::OAS_TASK_NODE_INFO_PATH, responses((status = 200, body = TaskNodeInfo)))]
pub async fn handler(
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
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
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task_node_info)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::tests,
        features::{
            task::Task,
            task_node::{self, routers::TASK_NODE_INFO_LIST_PATH, TaskNode},
        },
        AppResult, Db,
    };

    #[sqlx::test]
    async fn タスクノードを更新できる(db: Db) -> AppResult<()> {
        let task: Task = Default::default();
        let node = task_node::factory::create(
            &db,
            Some(TaskNode {
                task: task.clone(),
                node_info: TaskNodeInfo {
                    task_id: task.id,
                    x: 0.0,
                    y: 0.0,
                    ..TaskNode::default().node_info
                },
            }),
        )
        .await?;

        let new_x = 1.1;
        let new_y = -100.100;
        let server = tests::build(db.clone()).await?;
        server
            .put(&[TASK_NODE_INFO_LIST_PATH, &node.node_info.id].join("/"))
            .json(&UpdateTaskNodeInfo { x: new_x, y: new_y })
            .await;

        let updated = sqlx::query_as!(
            TaskNodeInfo,
            "SELECT * FROM task_node_info WHERE id = $1",
            node.node_info.id
        )
        .fetch_one(&db)
        .await?;

        assert_eq!(updated.x, new_x);
        assert_eq!(updated.y, new_y);

        Ok(())
    }
}
