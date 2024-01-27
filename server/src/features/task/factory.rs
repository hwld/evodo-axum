use crate::{AppResult, Db};

use super::*;

impl Default for Task {
    fn default() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Task {
            id,
            status: TaskStatus::Todo,
            title: "title".into(),
            created_at: "".into(),
            updated_at: "".into(),
        }
    }
}

pub async fn create(db: &Db, input: Option<Task>) -> AppResult<Task> {
    let task = input.unwrap_or_default();
    let created = sqlx::query_as!(
        Task,
        "INSERT INTO tasks(id, status, title) values($1, $2, $3) RETURNING *;",
        task.id,
        task.status,
        task.title
    )
    .fetch_one(db)
    .await?;

    Ok(created)
}
