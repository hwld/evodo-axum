use crate::{
    app::Connection,
    features::task::{
        db::{find_task, find_tasks, insert_task, FindTaskArgs, InsertTaskArgs},
        TaskStatus,
    },
};

use super::{TaskNode, TaskNodeInfo};

pub struct InsertTaskNodeArgs<'a> {
    pub task_id: &'a str,
    pub title: &'a str,
    pub status: &'a TaskStatus,
    pub user_id: &'a str,
    pub x: f64,
    pub y: f64,
}
pub async fn insert_task_node<'a>(
    db: &mut Connection,
    InsertTaskNodeArgs {
        task_id,
        title,
        status,
        user_id,
        x,
        y,
    }: InsertTaskNodeArgs<'a>,
) -> anyhow::Result<TaskNode> {
    let task = insert_task(
        db,
        InsertTaskArgs {
            id: task_id,
            title,
            description: "",
            user_id,
            status,
        },
    )
    .await?;

    let node_info = insert_task_node_info(
        db,
        InsertTaskNodeInfoArgs {
            task_id,
            user_id,
            x,
            y,
        },
    )
    .await?;

    Ok(TaskNode { task, node_info })
}

pub struct FindTaskNodeArgs<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn find_task_node<'a>(
    db: &mut Connection,
    FindTaskNodeArgs { task_id, user_id }: FindTaskNodeArgs<'a>,
) -> anyhow::Result<TaskNode> {
    let task = find_task(db, FindTaskArgs { task_id, user_id }).await?;
    let node_info = find_task_node_info(db, FindTaskNodeInfo { task_id, user_id }).await?;

    Ok(TaskNode { task, node_info })
}

pub async fn find_task_nodes<'a>(
    db: &mut Connection,
    user_id: &str,
) -> anyhow::Result<Vec<TaskNode>> {
    let tasks = find_tasks(db, user_id).await?;
    let node_info_list = find_task_node_info_list(db, user_id).await?;

    let mut result: Vec<TaskNode> = Vec::new();

    for task in tasks {
        let Some(node_info) = node_info_list
            .iter()
            .find(|i| i.task_id == task.id)
            .cloned()
        else {
            // taskがあるがtask_node_infoがない場合はスキップする
            continue;
        };

        let task_node = TaskNode { task, node_info };

        result.push(task_node);
    }

    Ok(result)
}

pub struct InsertTaskNodeInfoArgs<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
    pub x: f64,
    pub y: f64,
}
pub async fn insert_task_node_info<'a>(
    db: &mut Connection,
    InsertTaskNodeInfoArgs {
        task_id,
        user_id,
        x,
        y,
    }: InsertTaskNodeInfoArgs<'a>,
) -> anyhow::Result<TaskNodeInfo> {
    let result = sqlx::query!(
        r#" INSERT INTO task_node_info(task_id, user_id, x, y) VALUES($1, $2, $3, $4) RETURNING task_id;"#,
        task_id,
        user_id,
        x,
        y,
    ).fetch_one(&mut *db).await?;

    let task_node_info = find_task_node_info(
        db,
        FindTaskNodeInfo {
            task_id: &result.task_id,
            user_id,
        },
    )
    .await?;

    Ok(task_node_info)
}

pub struct UpdateTaskNodeInfoArgs<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
    pub x: f64,
    pub y: f64,
}
pub async fn update_task_node_info<'a>(
    db: &mut Connection,
    UpdateTaskNodeInfoArgs {
        task_id,
        user_id,
        x,
        y,
    }: UpdateTaskNodeInfoArgs<'a>,
) -> anyhow::Result<TaskNodeInfo> {
    let result = sqlx::query!(
        r#"
        UPDATE 
            task_node_info
        SET 
            x = $1,
            y = $2
        WHERE
            task_id = $3 AND user_id = $4
        RETURNING 
            task_id;
        "#,
        x,
        y,
        task_id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task_node_info = find_task_node_info(
        &mut *db,
        FindTaskNodeInfo {
            task_id: &result.task_id,
            user_id,
        },
    )
    .await?;

    Ok(task_node_info)
}

pub struct FindTaskNodeInfo<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn find_task_node_info<'a>(
    db: &mut Connection,
    FindTaskNodeInfo { task_id, user_id }: FindTaskNodeInfo<'a>,
) -> anyhow::Result<TaskNodeInfo> {
    let result = sqlx::query!(
        r#"SELECT * FROM task_node_info WHERE task_id = $1 AND user_id = $2"#,
        task_id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task_node_info = TaskNodeInfo {
        task_id: result.task_id,
        user_id: result.user_id,
        x: result.x,
        y: result.y,
    };
    Ok(task_node_info)
}

pub async fn find_task_node_info_list<'a>(
    db: &mut Connection,
    user_id: &str,
) -> anyhow::Result<Vec<TaskNodeInfo>> {
    let result = sqlx::query_as!(
        TaskNodeInfo,
        r#"SELECT * FROM task_node_info WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(result)
}
