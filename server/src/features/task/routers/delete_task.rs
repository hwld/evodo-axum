use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;

use crate::{features::task::Task, AppResult, AppState};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = super::TAG, path = super::OAS_TASK_PATH, responses((status = 200, body = Task)))]
pub async fn handler(
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let task = sqlx::query_as!(Task, r#"DELETE FROM tasks WHERE id = $1 RETURNING *"#, id)
        .fetch_one(&db)
        .await?;

    Ok((StatusCode::OK, Json(task)))
}

#[cfg(test)]
mod tests {
    use crate::{
        app::tests,
        features::task::{self, routers::TASKS_PATH},
        Db,
    };

    use super::*;

    #[sqlx::test]
    async fn タスクを削除できる(db: Db) -> AppResult<()> {
        let created_task = task::factory::create(&db, None).await?;

        let server = tests::build(db.clone()).await?;
        server
            .delete(&[TASKS_PATH, &created_task.id].join("/"))
            .await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn 存在しないタスクを削除しようとしても何も変わらない(
        db: Db,
    ) -> AppResult<()> {
        task::factory::create(&db, None).await?;

        let server = tests::build(db.clone()).await?;
        server.delete(&[TASKS_PATH, &"not"].join("/")).await;

        let tasks = sqlx::query!("SELECT * FROM tasks;").fetch_all(&db).await?;
        assert_eq!(tasks.len(), 1);

        Ok(())
    }
}
