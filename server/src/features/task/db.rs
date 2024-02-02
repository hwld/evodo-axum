use crate::app::{AppResult, Connection};

use super::ConnectSubtask;

/// タスク同士が循環接続になりうるかを判定する
pub async fn detect_circular_connection(
    db: &mut Connection,
    ConnectSubtask {
        parent_task_id,
        subtask_id,
    }: ConnectSubtask,
) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT subtask_id, parent_task_id
            FROM subtask_connections
            WHERE subtask_id = $1

            UNION

            SELECT s.subtask_id, s.parent_task_id
            FROM subtask_connections s
            JOIN ancestors a ON s.subtask_id = a.parent_task_id
        )

        SELECT DISTINCT parent_task_id
        FROM ancestors
        WHERE parent_task_id = $2
        "#,
        parent_task_id,
        subtask_id
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}
