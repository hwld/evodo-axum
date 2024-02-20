use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task_node::db::{find_task_node, FindTaskNodeArgs},
    },
};

#[tracing::instrument(err)]
#[utoipa::path(
    get,
    tag = super::TAG,
    path = super::TaskNodePaths::task_node_open_api(),
    responses((status = 200, body = TaskNode)),
    params(("id" = String, Path,))
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let task_node = find_task_node(
        &mut tx,
        FindTaskNodeArgs {
            task_id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(task_node)))
}
