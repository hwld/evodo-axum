use uuid::Uuid;

use crate::{AppResult, Db};

use super::*;

pub async fn create(db: &Db, input: Option<CreateTask>) -> AppResult<Task> {
    let uuid = Uuid::new_v4().to_string();
    let title = input.map(|i| i.title).unwrap_or("title".into());

    let task = sqlx::query_as!(
        Task,
        "INSERT INTO tasks(id, title) VALUES ($1, $2) RETURNING *",
        uuid,
        title,
    )
    .fetch_one(db)
    .await?;

    Ok(task)
}
