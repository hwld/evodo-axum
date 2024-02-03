use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::features::task_node::db::find_task_node_with_ancestors_list;
use crate::{app::AppState, error::AppError, features::auth::Auth};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TaskNodePaths::task_nodes(), responses((status = 200, body = [TaskNodeWithAncestors])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let result = find_task_node_with_ancestors_list(&mut tx, &user.id).await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(result)).into_response())
}

#[cfg(test)]
mod tests {

    use crate::app::AppResult;
    use crate::features::task_node::TaskNodeWithAncestors;
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

        let tasks: Vec<TaskNodeWithAncestors> =
            test.server().get(&TaskNodePaths::task_nodes()).await.json();
        assert_eq!(tasks.len(), 3);

        Ok(())
    }

    #[sqlx::test]
    async fn 全てのタスクの祖先タスクを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let node1 = task_node_factory::create_with_user(&db, &user.id).await?;
        let node2 = task_node_factory::create_subnode(&db, &user.id, &node1.task.id).await?;
        let node3 = task_node_factory::create_subnode(&db, &user.id, &node2.task.id).await?;
        let node4 = task_node_factory::create_subnode(&db, &user.id, &node3.task.id).await?;

        let nodes: Vec<TaskNodeWithAncestors> =
            test.server().get(&TaskNodePaths::task_nodes()).await.json();
        assert_eq!(nodes.len(), 4);

        let n4 = nodes.iter().find(|n| n.task.id == node4.task.id).unwrap();
        assert_eq!(n4.ancestor_task_ids.len(), 3);
        assert!([&node1.task.id, &node2.task.id, &node3.task.id,]
            .iter()
            .all(|i| n4.ancestor_task_ids.contains(i)));

        let n3 = nodes.iter().find(|n| n.task.id == node3.task.id).unwrap();
        assert_eq!(n3.ancestor_task_ids.len(), 2);
        assert!([&node1.task.id, &node2.task.id]
            .iter()
            .all(|i| n3.ancestor_task_ids.contains(i)));

        let n2 = nodes.iter().find(|n| n.task.id == node2.task.id).unwrap();
        assert_eq!(n2.ancestor_task_ids.len(), 1);
        assert!([&node1.task.id]
            .iter()
            .all(|i| n2.ancestor_task_ids.contains(i)));

        let n1 = nodes.iter().find(|n| n.task.id == node1.task.id).unwrap();
        assert_eq!(n1.ancestor_task_ids.len(), 0);

        Ok(())
    }
}
