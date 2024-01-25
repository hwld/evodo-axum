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
    use crate::{features::task_node, AppResult, Db};

    #[sqlx::test]
    async fn タスクノードを更新できる(db: Db) -> AppResult<()> {
        let node = task_node::factory::create(&db, None).await?;

        let new_x = 1.1;
        let new_y = -100.100;
        let _ = handler(
            Path(node.node_info.id.clone()),
            State(AppState { db: db.clone() }),
            Json(UpdateTaskNodeInfo { x: new_x, y: new_y }),
        )
        .await?;

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
