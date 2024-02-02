use crate::app::{AppResult, Connection};

use super::TaskNodeInfo;

pub struct InsertTaskNodeInfoArgs<'a> {
    pub id: &'a str,
    pub task_id: &'a str,
    pub user_id: &'a str,
    pub x: f64,
    pub y: f64,
}
pub async fn insert_task_node_info<'a>(
    db: &mut Connection,
    InsertTaskNodeInfoArgs {
        id,
        task_id,
        user_id,
        x,
        y,
    }: InsertTaskNodeInfoArgs<'a>,
) -> AppResult<TaskNodeInfo> {
    let result = sqlx::query!(
        r#" INSERT INTO task_node_info(id, task_id, user_id, x, y) VALUES($1, $2, $3, $4, $5) RETURNING id"#,
        id,
        task_id,
        user_id,
        x,
        y,
    ).fetch_one(&mut *db).await?;

    let task_node_info = find_task_node_info(
        db,
        FindTaskNodeInfo {
            id: &result.id,
            user_id,
        },
    )
    .await?;

    Ok(task_node_info)
}

pub struct UpdateTaskNodeInfoArgs<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub x: f64,
    pub y: f64,
}
pub async fn update_task_node_info<'a>(
    db: &mut Connection,
    UpdateTaskNodeInfoArgs { id, user_id, x, y }: UpdateTaskNodeInfoArgs<'a>,
) -> AppResult<TaskNodeInfo> {
    let result = sqlx::query!(
        r#"
        UPDATE 
            task_node_info
        SET 
            x = $1,
            y = $2
        WHERE
            id = $3 AND user_id = $4
        RETURNING 
            id;
        "#,
        x,
        y,
        id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task_node_info = find_task_node_info(
        &mut *db,
        FindTaskNodeInfo {
            id: &result.id,
            user_id,
        },
    )
    .await?;

    Ok(task_node_info)
}

pub struct FindTaskNodeInfo<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}
pub async fn find_task_node_info<'a>(
    db: &mut Connection,
    FindTaskNodeInfo { id, user_id }: FindTaskNodeInfo<'a>,
) -> AppResult<TaskNodeInfo> {
    let result = sqlx::query!(
        r#"SELECT * FROM task_node_info WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task_node_info = TaskNodeInfo {
        id: result.id,
        task_id: result.task_id,
        user_id: result.user_id,
        // TODO
        subnode_ids: vec![],
        ancestor_ids: vec![],
        x: result.x,
        y: result.y,
    };
    Ok(task_node_info)
}
