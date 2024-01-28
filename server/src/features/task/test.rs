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

    pub async fn create(db: &Db, task: Task) -> AppResult<Task> {
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

    pub async fn create_with_user(db: &Db, user_id: &str) -> AppResult<Task> {
        let task = Task {
            user_id: user_id.into(),
            ..Default::default()
        };
        create(db, task).await
    }
}

#[cfg(test)]
pub mod routers {
    use crate::features::task;

    impl task::routers::Paths {
        pub fn one_task(id: &str) -> String {
            Self::tasks() + "/" + id
        }
    }
}
