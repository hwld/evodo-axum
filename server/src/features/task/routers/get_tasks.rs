use axum::{extract::State, Json};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{features::task::Task, AppResult};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = "task", path = "/tasks", responses((status = 200, body = [Task])))]
pub async fn handler(State(pool): State<Pool<Sqlite>>) -> AppResult<(StatusCode, Json<Vec<Task>>)> {
    let tasks = sqlx::query_as!(Task, r#"select * from tasks;"#)
        .fetch_all(&pool)
        .await?;

    Ok((StatusCode::OK, Json(tasks)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn 全てのタスクを取得できる(pool: Pool<Sqlite>) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO tasks(id, title) VALUES
                ('1', 'title'),
                ('2', 'title'),
                ('3', 'title');
            "#
        )
        .execute(&pool)
        .await?;

        let _ = handler(State(pool.clone())).await?;

        let tasks = sqlx::query_as!(Task, "SELECT * FROM tasks;")
            .fetch_all(&pool)
            .await?;

        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
