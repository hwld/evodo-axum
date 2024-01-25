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
    use crate::{features::*, Db};

    use super::*;

    #[sqlx::test]
    async fn タスクを削除できる(db: Db) -> AppResult<()> {
        let created_task = task::factory::create(&db, None).await?;

        let _ = handler(Path(created_task.id), State(AppState { db: db.clone() })).await?;

        let tasks = sqlx::query!("select * from tasks;").fetch_all(&db).await?;

        assert_eq!(tasks.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn 存在しないタスクを削除しようとするとエラーが出る(db: Db) {
        let r = handler(Path("not".into()), State(AppState { db: db.clone() })).await;

        assert!(r.is_err());
    }
}
