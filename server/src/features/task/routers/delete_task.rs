use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    features::{auth::Auth, task::Task},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::Paths::oas_task(), responses((status = 200, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let task = sqlx::query_as!(
        Task,
        r#"DELETE FROM tasks WHERE id = $1 AND user_id = $2 RETURNING *;"#,
        id,
        user.id
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::tests,
        features::{
            auth::{self, routers::signup::CreateUser},
            task::{self, routers::Paths},
            user::{self, User},
        },
        Db,
    };

    use super::*;

    #[sqlx::test]
    async fn 指定したタスクだけを削除できる(db: Db) -> AppResult<()> {
        let mut server = tests::build(db.clone()).await?;
        server.do_save_cookies();

        let user: User = server
            .post(&auth::test::routes::Paths::test_login())
            .json(&CreateUser::default())
            .await
            .json();

        task::test::factory::create(&db, user.clone().id, None).await?;
        let created_task = task::test::factory::create(&db, user.id, None).await?;

        server
            .delete(&format!("{}/{}", Paths::tasks(), created_task.id))
            .await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人のタスクは削除できない(db: Db) -> AppResult<()> {
        let mut server = tests::build(db.clone()).await?;
        server.do_save_cookies();

        let other_user = user::test::factory::create(&db, Some(User::default())).await?;
        let other_user_task = task::test::factory::create(&db, other_user.id, None).await?;

        server
            .post(&auth::test::routes::Paths::test_login())
            .json(&CreateUser::default())
            .await;
        let res = server
            .delete(&format!("{}/{}", Paths::tasks(), other_user_task.id))
            .await;
        assert_ne!(res.status_code(), StatusCode::UNAUTHORIZED);

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }
}
