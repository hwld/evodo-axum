use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::AppResult;

use super::*;

pub async fn create(pool: &Pool<Sqlite>, input: Option<CreateTask>) -> AppResult<Task> {
    let uuid = Uuid::new_v4().to_string();
    let title = input.map(|i| i.title).unwrap_or("title".into());

    let task = sqlx::query_as!(
        Task,
        "INSERT INTO tasks(id, title) VALUES ($1, $2) RETURNING *",
        uuid,
        title,
    )
    .fetch_one(pool)
    .await?;

    Ok(task)
}
