use axum::{extract::State, Json};

use crate::{
    features::{self, task::CreateTask},
    AppResult, AppState, Db,
};

use super::{CreateTaskNode, TaskNode};

pub async fn create(db: &Db, input: Option<CreateTaskNode>) -> AppResult<TaskNode> {
    let (_, Json(task_node)) = features::task_node::routers::create_task_node::handler(
        State(AppState { db: db.clone() }),
        Json(
            input.unwrap_or(CreateTaskNode {
                x: 0.0,
                y: 0.0,
                task: CreateTask {
                    title: "title".into(),
                }
                .into(),
            }),
        ),
    )
    .await?;

    Ok(task_node)
}
