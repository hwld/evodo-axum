use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::features::task_node::db::find_task_nodes;
use crate::{app::AppState, error::AppError, features::auth::Auth};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TaskNodePaths::task_nodes(), responses((status = 200, body = [TaskNode])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let result = find_task_nodes(&mut tx, &user.id).await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(result)).into_response())
}

#[cfg(test)]
mod tests {

    use crate::app::AppResult;
    use crate::features::task_node::TaskNode;
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
