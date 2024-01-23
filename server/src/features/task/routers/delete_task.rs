use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{features::task::Task, AppResult};

#[tracing::instrument(err)]
#[utoipa::path(delete, tag = "task", path = "/tasks/{id}", responses((status = 200, body = Task)))]
pub async fn handler(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let task = sqlx::query_as!(Task, r#"DELETE FROM tasks WHERE id = $1 RETURNING *"#, id)
        .fetch_one(&pool)
        .await?;

    Ok((StatusCode::OK, Json(task)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn タスクを削除できる(pool: Pool<Sqlite>) -> AppResult<()> {
        let task_id = "task_id";
        sqlx::query!(
            "INSERT INTO tasks(id, title) VALUES($1, $2)",
            task_id,
            "title"
        )
        .execute(&pool)
        .await?;

        let _ = handler(Path(task_id.into()), State(pool.clone())).await?;

        let tasks = sqlx::query!("select * from tasks;")
            .fetch_all(&pool)
            .await?;

        assert_eq!(tasks.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn 存在しないタスクを削除しようとするとエラーが出る(
        pool: Pool<Sqlite>,
    ) {
        let r = handler(Path("not".into()), State(pool.clone())).await;

        assert!(r.is_err());
    }
}
