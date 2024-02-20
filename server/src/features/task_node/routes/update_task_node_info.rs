use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::AppResult,
    features::task_node::db::{update_task_node_info, UpdateTaskNodeInfoArgs},
};
use crate::{
    app::AppState,
    error::AppError,
    features::{auth::Auth, task_node::UpdateTaskNodeInfo},
};

#[tracing::instrument(err, skip_all)]
#[utoipa::path(
    put,
    tag = super::TAG,
    path = super::TaskNodePaths::task_node_info_open_api(),
    responses((status = 200, body = TaskNodeInfo)),
    params(("id" = String, Path,))
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<UpdateTaskNodeInfo>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let task_node_info = update_task_node_info(
        &mut tx,
        UpdateTaskNodeInfoArgs {
            task_id: &id,
            user_id: &user.id,
            x: payload.x,
            y: payload.y,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(task_node_info)).into_response())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::AppResult;
    use crate::features::task_node::db::{find_task_node_info, FindTaskNodeInfo};
    use crate::features::task_node::TaskNodeInfo;
    use crate::{
        app::{tests::AppTest, Db},
        features::{
            task::Task,
            task_node::{routes::TaskNodePaths, test::task_node_factory, TaskNode},
        },
    };

    #[sqlx::test]
    async fn タスクノードを更新できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task: Task = Task {
            user_id: user.clone().id,
            ..Default::default()
        };
        let TaskNode { node_info, .. } = task_node_factory::create(
            &db,
            TaskNode {
                task: task.clone(),
                node_info: TaskNodeInfo {
                    task_id: task.id,
                    user_id: user.clone().id,
                    ..Default::default()
                },
            },
        )
        .await?;

        let new_x = 1.1;
        let new_y = -100.100;
        let res = test
            .server()
            .put(&TaskNodePaths::one_task_node_info(&node_info.task_id))
            .json(&UpdateTaskNodeInfo { x: new_x, y: new_y })
            .await;
        res.assert_status_ok();

        let mut conn = db.acquire().await?;
        let updated = find_task_node_info(
            &mut conn,
            FindTaskNodeInfo {
                task_id: &node_info.task_id,
                user_id: &user.id,
            },
        )
        .await?;

        assert_eq!(updated.x, new_x);
        assert_eq!(updated.y, new_y);

        Ok(())
    }
}
