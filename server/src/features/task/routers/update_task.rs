use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::{Pool, Sqlite};

use crate::{
    features::task::{Task, UpdateTask},
    AppResult,
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = "task", path = "/tasks/{id}", responses((status = 200, body = Task)))]
pub async fn handler(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateTask>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let task = sqlx::query_as!(
        Task,
        r#"
            UPDATE
                tasks 
            SET
                status = $1,
                title = $2,
                updated_at = (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime'))
            WHERE
                id = $3 
            RETURNING *;
        "#,
        payload.status,
        payload.title,
        id,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::OK, Json(task)))
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use super::*;
    use crate::{
        features::task::{self, TaskStatus},
        AppResult,
    };

    #[sqlx::test]
    async fn タスクを更新できる(pool: Pool<Sqlite>) -> AppResult<()> {
        let task = task::factory::create(&pool, None).await?;

        let new_title = "new_title";
        let new_status = TaskStatus::Done;
        let _ = handler(
            Path(task.id.clone()),
            State(pool.clone()),
            Json(UpdateTask {
                title: new_title.into(),
                status: new_status,
            }),
        )
        .await?;

        let updated = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task.id)
            .fetch_one(&pool)
            .await?;

        assert_eq!(updated.title, new_title);
        assert_eq!(updated.status, new_status);

        Ok(())
    }
}
