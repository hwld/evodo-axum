use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    app::AppResult,
    features::task::{
        db::{delete_task, DeleteTaskArgs},
        DeleteTaskResponse,
    },
};
use crate::{app::AppState, error::AppError, features::auth::Auth};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::TaskPaths::task_open_api(), responses((status = 200, body = DeleteTaskResponse)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let deleted_id = delete_task(
        &mut tx,
        DeleteTaskArgs {
            id: &id,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(DeleteTaskResponse {
            task_id: deleted_id,
        }),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use crate::app::Db;
    use crate::features::task::test::task_factory;
    use crate::features::user::test::user_factory;
    use crate::{app::tests::AppTest, features::task::routes::TaskPaths};

    use super::*;

    #[sqlx::test]
    async fn 指定したタスクだけを削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        task_factory::create_with_user(&db, &user.id).await?;
        let created_task = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .delete(&TaskPaths::one_task(&created_task.id))
            .await;
        res.assert_status_ok();

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクは削除できない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;

        test.login(None).await?;
        let res = test
            .server()
            .delete(&TaskPaths::one_task(&other_user_task.id))
            .await;
        assert_ne!(res.status_code(), StatusCode::UNAUTHORIZED);

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }
}
