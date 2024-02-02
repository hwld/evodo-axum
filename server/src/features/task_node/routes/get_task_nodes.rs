use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::{
    app::AppState,
    error::AppError,
    features::{
        auth::Auth,
        task::Task,
        task_node::{TaskNode, TaskNodeInfo},
    },
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TaskNodePaths::task_nodes(), responses((status = 200, body = [TaskNode])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

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
        WHERE
            t.user_id = $1;
        "#,
        user.id,
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
                user_id: r.user_id.clone(),
                // TODO: 一旦空にするが、後でちゃんとsubtask_idsを入れる
                subtask_ids: Vec::new(),
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            node_info: TaskNodeInfo {
                task_id: r.task_id,
                user_id: r.user_id,
                // TODO:
                subnode_ids: vec![],
                ancestor_ids: vec![],
                x: r.x,
                y: r.y,
            },
        })
        .collect();

    Ok((StatusCode::OK, Json(nodes)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::AppResult;
    use crate::{
        app::{tests::AppTest, Db},
        features::task_node::{routes::TaskNodePaths, test::task_node_factory},
    };

    #[sqlx::test]
    async fn 全てのタスクノードを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        tokio::try_join!(
            task_node_factory::create_with_user(&db, &user.id),
            task_node_factory::create_with_user(&db, &user.id),
            task_node_factory::create_with_user(&db, &user.id)
        )?;

        let tasks: Vec<TaskNode> = test.server().get(&TaskNodePaths::task_nodes()).await.json();

        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
