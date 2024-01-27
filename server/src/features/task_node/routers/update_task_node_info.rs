use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    features::{
        auth::Auth,
        task_node::{TaskNodeInfo, UpdateTaskNodeInfo},
    },
    AppResult, AppState,
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
        return Ok(StatusCode::UNAUTHORIZED.into_response());
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
    use crate::{
        app::tests,
        features::{
            auth::{self, routers::signup::CreateUser},
            task::Task,
            task_node::{self, routers::Paths, TaskNode},
            user::User,
        },
        AppResult, Db,
    };

    #[sqlx::test]
    async fn タスクノードを更新できる(db: Db) -> AppResult<()> {
        let mut server = tests::build(db.clone()).await?;
        server.do_save_cookies();

        let user: User = server
            .post(&auth::test::routes::Paths::test_login())
            .json(&CreateUser::default())
            .await
            .json();

        let task: Task = Task {
            user_id: user.clone().id,
            ..Default::default()
        };
        let TaskNode { node_info, .. } = task_node::test::factory::create(
            &db,
            user.clone().id,
            Some(TaskNode {
                task: task.clone(),
                node_info: TaskNodeInfo {
                    task_id: task.id,
                    user_id: user.clone().id,
                    x: 0.0,
                    y: 0.0,
                    ..TaskNode::default().node_info
                },
            }),
        )
        .await?;

        let new_x = 1.1;
        let new_y = -100.100;
        server
            .put(&format!(
                "{}/{}",
                Paths::task_node_info_list(),
                node_info.id
            ))
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
