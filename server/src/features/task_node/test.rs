#[cfg(test)]
pub mod task_node_factory {
    use uuid::Uuid;

    use crate::app::Db;
    use crate::features::task::test::task_factory;
    use crate::features::task_node::db::{insert_task_node_info, InsertTaskNodeInfoArgs};
    use crate::{
        app::AppResult,
        features::{
            task::Task,
            task_node::{TaskNode, TaskNodeInfo},
        },
    };

    impl Default for TaskNode {
        fn default() -> Self {
            let task: Task = Default::default();
            let task_id = task.id.clone();

            TaskNode {
                task,
                node_info: TaskNodeInfo {
                    task_id,
                    ancestor_ids: vec![],
                    subnode_ids: vec![],
                    ..Default::default()
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
                subnode_ids: vec![],
                ancestor_ids: vec![],
            }
        }
    }

    pub async fn create(db: &Db, task_node: TaskNode) -> AppResult<TaskNode> {
        let task = task_factory::create(db, task_node.task).await?;

        let mut conn = db.acquire().await?;
        let node_info = insert_task_node_info(
            &mut conn,
            InsertTaskNodeInfoArgs {
                id: &task_node.node_info.id,
                task_id: &task.id,
                user_id: &task_node.node_info.user_id,
                x: task_node.node_info.x,
                y: task_node.node_info.y,
            },
        )
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

    impl task_node::routes::TaskNodePaths {
        pub fn one_task_node_info(id: &str) -> String {
            Self::task_node_info_list() + "/" + id
        }
    }
}
