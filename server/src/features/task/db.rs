use std::collections::HashMap;

use anyhow::anyhow;
use http::StatusCode;

use crate::{
    app::{AppResult, Connection},
    error::AppError,
};

use super::{Task, TaskAncestors, TaskStatus};

pub struct FindTaskArgs<'a> {
    pub user_id: &'a str,
    pub task_id: &'a str,
}
pub async fn find_task<'a>(
    db: &mut Connection,
    FindTaskArgs { user_id, task_id }: FindTaskArgs<'a>,
) -> AppResult<Task> {
    let raw_task = sqlx::query!(
        r#"
        SELECT t.*, s.parent_task_id, s.subtask_id
        FROM tasks t 
        LEFT OUTER JOIN subtask_connections s ON (t.id = s.parent_task_id AND t.user_id = s.user_id)
        WHERE t.user_id = $1 AND t.id = $2;
        "#,
        user_id,
        task_id,
    )
    .fetch_all(&mut *db)
    .await?;

    // 実際にはTaskは一つになるけど、いい感じに書く方法が思いつかなかったのでHashMapを使う
    let mut task_map: HashMap<String, Task> = HashMap::new();
    for raw in raw_task {
        let task = task_map.entry(raw.id.clone()).or_insert_with(|| Task {
            id: raw.id,
            title: raw.title,
            status: raw.status.into(),
            user_id: raw.user_id,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            subtask_ids: Vec::new(),
        });
        if let Some(subtask_id) = raw.subtask_id {
            task.subtask_ids.push(subtask_id);
        }
    }
    let (_, task) = task_map.into_iter().next().ok_or(anyhow!("Error"))?;

    Ok(task)
}

pub async fn find_tasks(db: &mut Connection, user_id: &str) -> AppResult<Vec<Task>> {
    let raw_tasks = sqlx::query!(
        r#"
        SELECT t.*, s.parent_task_id, s.subtask_id
        FROM tasks t 
        LEFT OUTER JOIN subtask_connections s ON (t.id = s.parent_task_id AND t.user_id = s.user_id)
        WHERE t.user_id = $1;
        "#,
        user_id
    )
    .fetch_all(&mut *db)
    .await?;

    let mut task_map: HashMap<String, Task> = HashMap::new();
    for raw in raw_tasks {
        let task = task_map.entry(raw.id.clone()).or_insert_with(|| Task {
            id: raw.id,
            title: raw.title,
            status: raw.status.into(),
            user_id: raw.user_id,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            subtask_ids: Vec::new(),
        });
        if let Some(subtask_id) = raw.subtask_id {
            task.subtask_ids.push(subtask_id);
        }
    }
    let tasks: Vec<Task> = task_map.into_values().collect();

    Ok(tasks)
}

pub struct InsertTaskArgs<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub user_id: &'a str,
    pub status: &'a TaskStatus,
}
pub async fn insert_task<'a>(
    db: &mut Connection,
    InsertTaskArgs {
        id,
        title,
        user_id,
        status,
    }: InsertTaskArgs<'a>,
) -> AppResult<Task> {
    let result = sqlx::query!(
        r#" INSERT INTO tasks(id, title, user_id, status) VALUES($1, $2, $3, $4) RETURNING *"#,
        id,
        title,
        user_id,
        status
    )
    .fetch_one(&mut *db)
    .await?;

    let task = find_task(
        &mut *db,
        FindTaskArgs {
            user_id,
            task_id: &result.id,
        },
    )
    .await?;

    Ok(task)
}

pub struct DeleteTaskArgs<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}
pub async fn delete_task<'a>(
    db: &mut Connection,
    DeleteTaskArgs { id, user_id }: DeleteTaskArgs<'a>,
) -> AppResult<String> {
    let result = sqlx::query!(
        r#"DELETE FROM tasks WHERE id = $1 AND user_id = $2 RETURNING *;"#,
        id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(result.id)
}

pub struct UpdateTaskArgs<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub status: &'a TaskStatus,
    pub user_id: &'a str,
}
pub async fn update_task<'a>(
    db: &mut Connection,
    UpdateTaskArgs {
        id,
        title,
        status,
        user_id,
    }: UpdateTaskArgs<'a>,
) -> AppResult<Task> {
    let result = sqlx::query!(
        r#"
        UPDATE
            tasks 
        SET
            status = $1,
            title = $2,
            updated_at = (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime'))
        WHERE
            id = $3 AND user_id = $4
        RETURNING *;        
        "#,
        status,
        title,
        id,
        user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task = find_task(
        &mut *db,
        FindTaskArgs {
            user_id,
            task_id: &result.id,
        },
    )
    .await?;

    Ok(task)
}

pub struct InsertSubTaskConnectionArgs<'a> {
    pub parent_task_id: &'a str,
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn insert_subtask_connection<'a>(
    db: &mut Connection,
    args: InsertSubTaskConnectionArgs<'a>,
) -> AppResult<()> {
    // ログインユーザーが指定されたタスクを持っているかを確認する
    let tasks = sqlx::query!(
        "SELECT * FROM tasks WHERE id IN ($1, $2) AND user_id = $3;",
        args.parent_task_id,
        args.subtask_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    if tasks.len() != 2 {
        return Err(AppError::new(StatusCode::NOT_FOUND, None));
    }

    // タスク同士が循環していないかを確認する。
    // payload.parent_task_idの祖先に、payload.subtask_idを持つtaskが存在しないことを確認する。
    if detect_circular_connection(&mut *db, &args).await? {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("タスクの循環は許可されていません。"),
        ));
    }

    sqlx::query!(
        "INSERT INTO subtask_connections(parent_task_id, subtask_id, user_id) VALUES($1, $2, $3) RETURNING *;",
        args.parent_task_id,
        args.subtask_id,
        args.user_id,
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(())
}

pub struct DeleteSubTaskConnectionArgs<'a> {
    pub parent_task_id: &'a str,
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn delete_subtask_connection<'a>(
    db: &mut Connection,
    args: DeleteSubTaskConnectionArgs<'a>,
) -> AppResult<()> {
    sqlx::query!(
        "DELETE FROM subtask_connections WHERE parent_task_id = $1 AND subtask_id = $2 AND user_id = $3 RETURNING *",
        args.parent_task_id,
        args.subtask_id,
        args.user_id
    ).fetch_one(&mut *db).await?;

    Ok(())
}

/// タスク同士が循環接続になりうるかを判定する
pub async fn detect_circular_connection<'a>(
    db: &mut Connection,
    &InsertSubTaskConnectionArgs {
        parent_task_id,
        subtask_id,
        user_id,
    }: &InsertSubTaskConnectionArgs<'a>,
) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT subtask_id, parent_task_id
            FROM subtask_connections
            WHERE subtask_id = $1 AND user_id = $2

            UNION

            SELECT s.subtask_id, s.parent_task_id
            FROM subtask_connections s
            JOIN ancestors a ON s.subtask_id = a.parent_task_id
        )

        SELECT DISTINCT parent_task_id
        FROM ancestors
        WHERE parent_task_id = $3
        "#,
        parent_task_id,
        user_id,
        subtask_id,
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}

pub async fn find_task_ancestors_list(
    db: &mut Connection,
    user_id: &str,
) -> AppResult<Vec<TaskAncestors>> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT
                subtask_id,
                parent_task_id
            FROM 
                subtask_connections
            WHERE
                user_id = $1
            UNION
            SELECT
                a.subtask_id,
                s.parent_task_id
            FROM
                subtask_connections s
                JOIN ancestors a
                    ON s.subtask_id = a.parent_task_id
            WHERE
                user_id = $1
        )

        SELECT DISTINCT
            subtask_id as task_id,
            parent_task_id as ancestor_id
        FROM
            ancestors
        "#,
        user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    let mut task_ancestors_map: HashMap<String, TaskAncestors> = HashMap::new();
    for raw in result {
        let task_ancestors = task_ancestors_map
            .entry(raw.task_id.clone())
            .or_insert_with(|| TaskAncestors {
                task_id: raw.task_id,
                ancestor_task_ids: vec![],
            });
        task_ancestors.ancestor_task_ids.push(raw.ancestor_id);
    }
    let task_ancestors_list: Vec<TaskAncestors> = task_ancestors_map.into_values().collect();

    Ok(task_ancestors_list)
}
