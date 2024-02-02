use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;
use sqlx::Acquire;

use crate::app::{AppResult, AppState};
use crate::error::AppError;
use crate::features::auth::Auth;
use crate::features::task::Task;
use crate::features::task_node::{CreateTaskNode, TaskNode, TaskNodeInfo};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::TaskNodePaths::task_nodes(), request_body = CreateTaskNode, responses((status = 201, body = TaskNode)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateTaskNode>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, None));
    };

    let mut tx = db.begin().await?;
    let conn = tx.acquire().await?;

    let task_id = uuid::Uuid::new_v4().to_string();
    let task_input = &payload.task;
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, user_id, title) VALUES($1, $2, $3) RETURNING * "#,
        task_id,
        user.id,
        task_input.title,
    )
    .fetch_one(&mut *conn)
    .await?;

    let node_info_id = uuid::Uuid::new_v4().to_string();
    let node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#" INSERT INTO task_node_info(id, task_id, user_id, x, y) VALUES($1, $2, $3, $4, $5) RETURNING * "#,
        node_info_id,
        task.id,
        user.id,
        payload.x,
        payload.y
    )
    .fetch_one(&mut *conn)
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(TaskNode { task, node_info })).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppResult;
    use crate::{
        app::{tests::AppTest, Db},
        features::{task::CreateTask, task_node::routes::TaskNodePaths},
    };

    #[sqlx::test]
    async fn タスクノードを作成するとタスクとノード情報が作成される(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        test.login(None).await?;

        let task_title = "title";
        let node_x = 0.0;
        let node_y = 0.0;

        let task_node: TaskNode = test
            .server()
            .post(&TaskNodePaths::task_nodes())
            .json(&CreateTaskNode {
                x: node_x,
                y: node_y,
                task: CreateTask {
                    title: task_title.into(),
                },
            })
            .await
            .json();

        let task = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task_node.task.id)
            .fetch_one(&db)
            .await?;
        assert_eq!(task.title, task_title);

        let node_info = sqlx::query_as!(
            TaskNodeInfo,
            "SELECT * FROM task_node_info WHERE id = $1",
            task_node.node_info.id
        )
        .fetch_one(&db)
        .await?;
        assert_eq!(node_info.x, node_x);
        assert_eq!(node_info.y, node_y);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字列のタスクノードは作成できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        test.login(None).await?;

        let res = test
            .server()
            .post(&TaskNodePaths::task_nodes())
            .json(&CreateTaskNode {
                task: CreateTask { title: "".into() },
                x: 0.0,
                y: -100.0,
            })
            .await;
        assert_ne!(res.status_code(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
