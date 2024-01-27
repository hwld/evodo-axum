use axum::{extract::State, Json};
use http::StatusCode;

use crate::{
    features::{
        task::Task,
        task_node::{TaskNode, TaskNodeInfo},
    },
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TASK_NODES_PATH, responses((status = 200, body = [TaskNode])))]
pub async fn handler(
    State(AppState { db }): State<AppState>,
) -> AppResult<(StatusCode, Json<Vec<TaskNode>>)> {
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
    .fetch_all(&db)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        app::tests,
        features::task_node::{self, routers::TASK_NODES_PATH},
        AppResult, Db,
    };

    #[sqlx::test]
    async fn 全てのタスクノードを取得できる(db: Db) -> AppResult<()> {
        tokio::try_join!(
            task_node::factory::create(&db, None),
            task_node::factory::create(&db, None),
            task_node::factory::create(&db, None)
        )?;

        let server = tests::build(db.clone()).await?;
        let tasks: Vec<TaskNode> = server.get(TASK_NODES_PATH).await.json();

        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
