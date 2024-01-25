use axum::{extract::State, Json};
use http::StatusCode;

use crate::{features::task::Task, AppResult, AppState};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = "task", path = "/tasks", responses((status = 200, body = [Task])))]
pub async fn handler(
    State(AppState { db }): State<AppState>,
) -> AppResult<(StatusCode, Json<Vec<Task>>)> {
    let tasks = sqlx::query_as!(Task, r#"select * from tasks;"#)
        .fetch_all(&db)
        .await?;

    Ok((StatusCode::OK, Json(tasks)))
}

#[cfg(test)]
mod tests {
    use crate::{features::task, Db};

    use super::*;

    #[sqlx::test]
    async fn 全てのタスクを取得できる(db: Db) -> AppResult<()> {
        let _ = tokio::try_join!(
            task::factory::create(&db, None),
            task::factory::create(&db, None),
            task::factory::create(&db, None),
        )?;

        let _ = handler(State(AppState { db: db.clone() })).await?;

        let tasks = sqlx::query_as!(Task, "SELECT * FROM tasks;")
            .fetch_all(&db)
            .await?;

        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
