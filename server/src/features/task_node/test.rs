#[cfg(test)]
pub mod factory {
    use uuid::Uuid;

    use crate::{
        features::{
            task::{self, Task},
            task_node::{TaskNode, TaskNodeInfo},
        },
        AppResult, Db,
    };

    impl Default for TaskNode {
        fn default() -> Self {
            let id = uuid::Uuid::new_v4().to_string();
            let task: Task = Default::default();

            TaskNode {
                task: task.clone(),
                node_info: TaskNodeInfo {
                    id,
                    task_id: task.id,
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

    // TODO:user_idとinputのuser_idが重複しちゃう
    pub async fn create(db: &Db, user_id: String, input: Option<TaskNode>) -> AppResult<TaskNode> {
        let task_node = input.unwrap_or(TaskNode {
            task: Task {
                user_id: user_id.clone(),
                ..Default::default()
            },
            node_info: TaskNodeInfo {
                user_id: user_id.clone(),
                ..Default::default()
            },
        });

        let task = task::test::factory::create(db, user_id, Some(task_node.task)).await?;
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
}
