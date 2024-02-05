use axum::{
    extract::{Path, State},
    Json,
};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{
            db::{find_task, FindTaskArgs},
            Task,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TaskPaths::task_open_api(), responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<Json<Task>> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let task = find_task(
        &mut tx,
        FindTaskArgs {
            task_id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(Json(task))
}
