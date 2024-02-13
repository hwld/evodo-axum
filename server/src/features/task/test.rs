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
                description: "description".into(),
                sub_task_ids: Vec::new(),
                blocked_task_ids: Vec::new(),
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
                description: &task.description,
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

    pub async fn create_sub_task(db: &Db, parent_task_id: &str, task: Task) -> AppResult<Task> {
        create(db, task.clone()).await?;

        sqlx::query!(
            "INSERT INTO sub_tasks(main_task_id, sub_task_id, user_id) VALUES($1, $2, $3);",
            parent_task_id,
            task.id,
            task.user_id,
        )
        .execute(db)
        .await?;

        Ok(task)
    }

    pub async fn create_default_sub_task(
        db: &Db,
        user_id: &str,
        parent_task_id: &str,
    ) -> AppResult<Task> {
        let sub_task = Task {
            user_id: user_id.into(),
            ..Default::default()
        };

        let sub_task = create_sub_task(db, parent_task_id, sub_task).await?;
        Ok(sub_task)
    }

    pub async fn create_blocking_connection(
        db: &Db,
        user_id: &str,
        blocking_task_id: &str,
        blocked_task_id: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO blocking_tasks(blocking_task_id, blocked_task_id, user_id) VALUES($1, $2, $3);",
            blocking_task_id,
            blocked_task_id,
            user_id,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn create_blocked_task(
        db: &Db,
        blocking_task_id: &str,
        task: Task,
    ) -> AppResult<Task> {
        create(db, task.clone()).await?;

        create_blocking_connection(db, &task.user_id, blocking_task_id, &task.id).await?;

        Ok(task)
    }

    pub async fn create_default_blocked_task(
        db: &Db,
        user_id: &str,
        blocking_task_id: &str,
    ) -> AppResult<Task> {
        let blocked = Task {
            user_id: user_id.into(),
            ..Default::default()
        };

        let blocked = create_blocked_task(db, blocking_task_id, blocked).await?;
        Ok(blocked)
    }
}

#[cfg(test)]
pub mod routes {
    use crate::features::task;

    impl task::routes::TaskPaths {
        pub fn one_task(id: &str) -> String {
            Self::tasks() + "/" + id
        }
        pub fn one_update_task_status(id: &str) -> String {
            Self::one_task(id) + &Self::update_task_status_base()
        }
    }
}
