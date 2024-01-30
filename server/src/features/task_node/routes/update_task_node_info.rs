use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::{
    app::AppState,
    error::AppError,
    features::{
        auth::Auth,
        task_node::{TaskNodeInfo, UpdateTaskNodeInfo},
    },
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::Paths::oas_task_node_info(), responses((status = 200, body = TaskNodeInfo)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<UpdateTaskNodeInfo>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, None));
    };

    let task_node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#"
        UPDATE
            task_node_info
        SET
            x = $1,
            y = $2
        WHERE
            id = $3 AND user_id = $4
        RETURNING *;
        "#,
        payload.x,
        payload.y,
        id,
        user.id,
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task_node_info)).into_response())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::AppResult;
    use crate::{
        app::{tests::AppTest, Db},
        features::{
            task::Task,
            task_node::{routes::Paths, test::factory as task_node_factory, TaskNode},
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
                    x: 0.0,
                    y: 0.0,
                    ..TaskNode::default().node_info
                },
            },
        )
        .await?;

        let new_x = 1.1;
        let new_y = -100.100;
        test.server()
            .put(&Paths::one_task_node_info(&node_info.id))
            .json(&UpdateTaskNodeInfo { x: new_x, y: new_y })
            .await;

        let updated = sqlx::query_as!(
            TaskNodeInfo,
            "SELECT * FROM task_node_info WHERE id = $1",
            node_info.id
        )
        .fetch_one(&db)
        .await?;

        assert_eq!(updated.x, new_x);
        assert_eq!(updated.y, new_y);

        Ok(())
    }
}
