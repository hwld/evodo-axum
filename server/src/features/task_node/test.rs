#[cfg(test)]
pub mod factory {
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
                    x: 0.0,
                    y: 0.0,
                },
            }
        }
    }

    pub async fn create(db: &Db, input: Option<TaskNode>) -> AppResult<TaskNode> {
        let task_node = input.unwrap_or_default();

        let task = task::test::factory::create(db, Some(task_node.task)).await?;
        let node_info = sqlx::query_as!(
            TaskNodeInfo,
            "INSERT INTO task_node_info(id, task_id, x, y) VALUES($1, $2, $3, $4) RETURNING *;",
            task_node.node_info.id,
            task.id,
            task_node.node_info.x,
            task_node.node_info.y
        )
        .fetch_one(db)
        .await?;

        Ok(TaskNode { task, node_info })
    }
}
