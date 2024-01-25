use axum::{extract::State, Json};

use crate::{features, AppResult, AppState, Db};

use super::*;

pub async fn create(db: &Db, input: Option<CreateTask>) -> AppResult<Task> {
    let (_, Json(task)) = features::task::routers::create_task::handler(
        State(AppState { db: db.clone() }),
        Json(
            input
                .unwrap_or(CreateTask {
                    title: "title".into(),
                })
                .into(),
        ),
    )
    .await?;

    Ok(task)
}
