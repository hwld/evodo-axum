use axum::{extract::State, Json};
use http::StatusCode;

use crate::{
    features::task::{CreateTask, Task},
    AppResult, Db,
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = "task", path = "/tasks", responses((status = 201, body = Task)))]
pub async fn handler(
    State(db): State<Db>,
    Json(payload): Json<CreateTask>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING *"#,
        uuid,
        payload.title,
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn タスクを作成できる(db: Db) -> AppResult<()> {
        let title = "title";

        let (_, task) = handler(
            State(db.clone()),
            Json(CreateTask {
                title: title.into(),
            }),
        )
        .await?;

        let created = sqlx::query_as!(Task, "select * from tasks where id = $1", task.id)
            .fetch_all(&db)
            .await?;
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, title);

        Ok(())
    }
}
