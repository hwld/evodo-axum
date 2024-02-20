use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::{AppResult, AppState};
use crate::error::AppError;
use crate::features::auth::Auth;
use crate::features::task_node::db::{insert_task_node, InsertTaskNodeArgs};
use crate::features::task_node::CreateTaskNode;

#[tracing::instrument(err)]
#[utoipa::path(
    post,
    tag = super::TAG,
    path = super::TaskNodePaths::task_nodes(),
    request_body = CreateTaskNode, responses((status = 201, body = TaskNode))
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateTaskNode>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let task_id = uuid::Uuid::new_v4().to_string();
    let task_node = insert_task_node(
        &mut tx,
        InsertTaskNodeArgs {
            task_id: &task_id,
            title: &payload.task.title,
            status: &Default::default(),
            user_id: &user.id,
            x: payload.x,
            y: payload.y,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(task_node)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppResult;
    use crate::features::task_node::db::{find_task_node, FindTaskNodeArgs};
    use crate::features::task_node::TaskNode;
    use crate::{
        app::{tests::AppTest, Db},
        features::{task::CreateTask, task_node::routes::TaskNodePaths},
    };

    #[sqlx::test]
    async fn タスクノードを作成するとタスクとノード情報が作成される(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

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

        let mut conn = db.acquire().await?;
        let created = find_task_node(
            &mut conn,
            FindTaskNodeArgs {
                task_id: &task_node.task.id,
                user_id: &user.id,
            },
        )
        .await?;
        assert_eq!(created.task.title, task_title);
        assert_eq!(created.node_info.x, node_x);
        assert_eq!(created.node_info.y, node_y);

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
