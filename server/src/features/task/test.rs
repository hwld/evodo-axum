#[cfg(test)]
pub mod task_factory {
    use uuid::Uuid;

    use crate::app::AppResult;
    use crate::features::task::db::{insert_task, InsertTaskArgs};
    use crate::{app::Db, features::task::Task};

    impl Default for Task {
        fn default() -> Self {
            Task {
                id: Uuid::new_v4().into(),
                status: Default::default(),
                user_id: "user_id".into(),
                title: "title".into(),
                subtask_ids: Vec::new(),
                created_at: "".into(),
                updated_at: "".into(),
            }
        }
    }

    pub async fn create(db: &Db, task: Task) -> AppResult<Task> {
        let mut conn = db.acquire().await?;
        let created = insert_task(
            &mut conn,
            InsertTaskArgs {
                id: &task.id,
                title: &task.title,
                user_id: &task.user_id,
                status: &task.status,
            },
        )
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

    pub async fn create_subatsk(db: &Db, user_id: &str, task_id: &str) -> AppResult<Task> {
        let subtask = Task {
            user_id: user_id.into(),
            ..Default::default()
        };
        create(db, subtask.clone()).await?;

        sqlx::query!(
            "INSERT INTO subtask_connections(parent_task_id, subtask_id, user_id) VALUES($1, $2, $3);",
            task_id,
            subtask.id,
            user_id,
        )
        .execute(db)
        .await?;

        Ok(subtask)
    }
}

#[cfg(test)]
pub mod routes {
    use crate::features::task;

    impl task::routes::TaskPaths {
        pub fn one_task(id: &str) -> String {
            Self::tasks() + "/" + id
        }
    }
}
