#[cfg(test)]
pub mod factory {

    use uuid::Uuid;

    use crate::{
        features::task::{Task, TaskStatus},
        AppResult, Db,
    };

    impl Default for Task {
        fn default() -> Self {
            Task {
                id: Uuid::new_v4().into(),
                status: TaskStatus::Todo,
                user_id: "user_id".into(),
                title: "title".into(),
                created_at: "".into(),
                updated_at: "".into(),
            }
        }
    }

    // TODO:user_idとinputのuser_idが重複しちゃう
    pub async fn create(db: &Db, user_id: String, input: Option<Task>) -> AppResult<Task> {
        let task = input.unwrap_or(Task {
            user_id,
            ..Default::default()
        });

        let created = sqlx::query_as!(
            Task,
            // user_idが存在しないときにはエラーになる
            "INSERT INTO tasks(id, status, title, user_id) values($1, $2, $3, $4) RETURNING *;",
            task.id,
            task.status,
            task.title,
            task.user_id,
        )
        .fetch_one(db)
        .await?;

        Ok(created)
    }
}
