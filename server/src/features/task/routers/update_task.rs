use axum::{
    extract::{Path, State},
    Json,
};
use garde::Unvalidated;
use http::StatusCode;

use crate::{
    features::task::{Task, UpdateTask},
    AppResult, AppState,
};

#[tracing::instrument(err)]
#[utoipa::path(put, tag = super::TAG, path = super::OAS_TASK_PATH, request_body = UpdateTask, responses((status = 200, body = Task)))]
pub async fn handler(
    Path(id): Path<String>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<Unvalidated<UpdateTask>>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let input = payload.validate(&())?;

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
        input.status,
        input.title,
        id,
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(task)))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        features::task::{self, TaskStatus},
        AppResult, Db,
    };

    #[sqlx::test]
    async fn タスクを更新できる(db: Db) -> AppResult<()> {
        let task = task::factory::create(&db, None).await?;

        let new_title = "new_title";
        let new_status = TaskStatus::Done;
        let _ = handler(
            Path(task.id.clone()),
            State(AppState { db: db.clone() }),
            Json(
                UpdateTask {
                    title: new_title.into(),
                    status: new_status,
                }
                .into(),
            ),
        )
        .await?;

        let updated = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task.id)
            .fetch_one(&db)
            .await?;

        assert_eq!(updated.title, new_title);
        assert_eq!(updated.status, new_status);

        Ok(())
    }

    #[sqlx::test]
    async fn 空文字列には更新できない(db: Db) -> AppResult<()> {
        let task = task::factory::create(&db, None).await?;

        let result = handler(
            Path(task.id),
            State(AppState { db: db.clone() }),
            Json(
                UpdateTask {
                    title: "".into(),
                    status: TaskStatus::Done,
                }
                .into(),
            ),
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
