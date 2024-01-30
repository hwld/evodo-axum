#[cfg(test)]
pub mod factory {
    use uuid::Uuid;

    use crate::app::Db;
    use crate::features::task::test::factory as task_factory;
    use crate::{
        app::AppResult,
        features::{
            task::Task,
            task_node::{TaskNode, TaskNodeInfo},
        },
    };

    impl Default for TaskNode {
        fn default() -> Self {
            let id = uuid::Uuid::new_v4().to_string();
            let task: Task = Default::default();
            let task_id = task.id.clone();

            TaskNode {
                task,
                node_info: TaskNodeInfo {
                    id,
                    task_id,
                    user_id: "user_id".into(),
                    x: 0.0,
                    y: 0.0,
                },
            }
        }
    }

    impl Default for TaskNodeInfo {
        fn default() -> Self {
            TaskNodeInfo {
                id: Uuid::new_v4().into(),
                task_id: "task_id".into(),
                user_id: "user_id".into(),
                x: 0.0,
                y: 0.0,
            }
        }
    }

    pub async fn create(db: &Db, task_node: TaskNode) -> AppResult<TaskNode> {
        let task = task_factory::create(db, task_node.task).await?;
        let node_info = sqlx::query_as!(
            TaskNodeInfo,
            "INSERT INTO task_node_info(id, task_id, user_id, x, y) VALUES($1, $2, $3, $4, $5) RETURNING *;",
            task_node.node_info.id,
            task.id,
            task_node.node_info.user_id,
            task_node.node_info.x,
            task_node.node_info.y
        )
        .fetch_one(db)
        .await?;

        Ok(TaskNode { task, node_info })
    }

    pub async fn create_with_user(db: &Db, user_id: &str) -> AppResult<TaskNode> {
        let task_node = TaskNode {
            task: Task {
                user_id: user_id.into(),
                ..Default::default()
            },
            node_info: TaskNodeInfo {
                user_id: user_id.into(),
                ..Default::default()
            },
        };

        create(db, task_node).await
    }
}

#[cfg(test)]
pub mod routes {
    use crate::features::task_node;

    impl task_node::routes::Paths {
        pub fn one_task_node_info(id: &str) -> String {
            Self::task_node_info_list() + "/" + id
        }
    }
}
