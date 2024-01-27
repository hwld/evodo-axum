use axum::{extract::State, Json};
use http::StatusCode;

use crate::{features::task::Task, AppResult, AppState};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TASKS_PATH, responses((status = 200, body = [Task])))]
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
    use crate::{
        app::tests,
        features::task::{self, routers::TASKS_PATH},
        Db,
    };

    use super::*;

    #[sqlx::test]
    async fn 全てのタスクを取得できる(db: Db) -> AppResult<()> {
        tokio::try_join!(
            task::factory::create(&db, None),
            task::factory::create(&db, None),
            task::factory::create(&db, None),
        )?;

        let server = tests::build(db.clone()).await?;
        let tasks: Vec<Task> = server.get(TASKS_PATH).await.json();

        assert_eq!(tasks.len(), 3);

        Ok(())
    }
}
