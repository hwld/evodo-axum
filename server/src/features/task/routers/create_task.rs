use axum::{extract::State, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_login::AuthSession;
use http::StatusCode;

use crate::{
    features::{
        auth::Auth,
        task::{CreateTask, Task},
    },
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::Paths::tasks(), request_body = CreateTask, responses((status = 201, body = Task)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateTask>>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title, user_id) VALUES($1, $2, $3) RETURNING *"#,
        uuid,
        payload.title,
        user.id,
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(task)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::tests,
        features::{
            auth::{self, routers::signup::CreateUser},
            task::{routers::Paths, CreateTask, Task},
        },
        AppResult, Db,
    };

    #[sqlx::test]
    async fn タスクを作成できる(db: Db) -> AppResult<()> {
        let mut server = tests::build(db.clone()).await?;
        server.do_save_cookies();

        server
            .post(&auth::test::routes::Paths::test_login())
            .json(&CreateUser::default())
            .await;

        let title = "title";
        let res_task: Task = server
            .post(&Paths::tasks())
            .json(&CreateTask {
                title: title.into(),
            })
            .await
            .json();

        let created = sqlx::query_as!(Task, "SELECT * FROM tasks where id = $1", res_task.id)
            .fetch_all(&db)
            .await?;
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, title);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字のタスクを作成できない(db: Db) -> AppResult<()> {
        let mut server = tests::build(db.clone()).await?;
        server.do_save_cookies();

        server
            .post(&auth::test::routes::Paths::test_login())
            .json(&CreateUser::default())
            .await;

        server
            .post(&Paths::tasks())
            .json(&CreateTask { title: "".into() })
            .await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 0);
        Ok(())
    }
}
