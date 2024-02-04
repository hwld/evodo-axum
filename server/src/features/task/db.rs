use std::collections::HashMap;

use anyhow::anyhow;
use async_recursion::async_recursion;
use http::StatusCode;
use sqlx::{QueryBuilder, Row, Sqlite};

use crate::{
    app::{AppResult, Connection},
    error::AppError,
};

use super::{Task, TaskStatus};

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

pub struct FindParentTaskArgs<'a> {
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn find_parent_tasks<'a>(
    db: &mut Connection,
    args: FindParentTaskArgs<'a>,
) -> AppResult<Vec<Task>> {
    let raw_parent_tasks = sqlx::query!(
        r#"
        SELECT t.*, sc.subtask_id
        FROM subtask_connections sc
        LEFT OUTER JOIN tasks t ON (t.id = sc.parent_task_id AND t.user_id = sc.user_id)
        WHERE sc.subtask_id = $1 AND t.user_id = $2;
        "#,
        args.subtask_id,
        args.user_id
    )
    .fetch_all(&mut *db)
    .await?;

    let mut task_map: HashMap<String, Task> = HashMap::new();
    for raw_task in raw_parent_tasks {
        let task = task_map.entry(raw_task.id.clone()).or_insert(Task {
            id: raw_task.id,
            status: raw_task.status.into(),
            title: raw_task.title,
            user_id: raw_task.user_id,
            subtask_ids: Vec::new(),
            created_at: raw_task.created_at,
            updated_at: raw_task.updated_at,
        });
        task.subtask_ids.push(raw_task.subtask_id);
    }

    let parent_tasks: Vec<Task> = task_map.into_values().collect();
    Ok(parent_tasks)
}

pub async fn all_tasks_done(db: &mut Connection, task_ids: &Vec<String>) -> AppResult<bool> {
    if task_ids.is_empty() {
        return Ok(true);
    }

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT count(*)
        FROM tasks
        WHERE status <> 'Done' AND id IN (",
    );

    let mut separated = query_builder.separated(", ");
    for id in task_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    let query = query_builder.build();
    let result = query.fetch_one(&mut *db).await?;

    let count: i32 = result.get(0);
    Ok(count == 0)
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

pub struct UpdateTaskStatusArgs<'a> {
    pub id: &'a str,
    pub status: &'a TaskStatus,
    pub user_id: &'a str,
}
pub async fn update_task_status<'a>(
    db: &mut Connection,
    args: UpdateTaskStatusArgs<'a>,
) -> AppResult<Task> {
    let result = sqlx::query!(
        r#"
        UPDATE
            tasks 
        SET
            status = $1,
            updated_at = (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime'))
        WHERE
            id = $2 AND user_id = $3
        RETURNING *;        
        "#,
        args.status,
        args.id,
        args.user_id
    )
    .fetch_one(&mut *db)
    .await?;

    let task = find_task(
        &mut *db,
        FindTaskArgs {
            user_id: args.user_id,
            task_id: &result.id,
        },
    )
    .await?;

    Ok(task)
}

pub struct TaskAndUser<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
#[async_recursion]
pub async fn update_all_ancestors_task_status<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> AppResult<()>
where
    'a: 'async_recursion,
{
    let parents = find_parent_tasks(
        &mut *db,
        FindParentTaskArgs {
            subtask_id: args.task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if parents.is_empty() {
        return Ok(());
    };

    for parent in parents {
        let all_subtasks_done = all_tasks_done(&mut *db, &parent.subtask_ids).await?;
        let new_status = if all_subtasks_done {
            TaskStatus::Done
        } else {
            TaskStatus::Todo
        };

        // 親が多いことを想定するならupdateをまとめたほうがいいかもしれないけど、多くならないと思う
        update_task_status(
            &mut *db,
            UpdateTaskStatusArgs {
                id: &parent.id,
                status: &new_status,
                user_id: args.user_id,
            },
        )
        .await?;

        // 親の祖先も同じように処理する
        update_all_ancestors_task_status(
            &mut *db,
            TaskAndUser {
                task_id: &parent.id,
                user_id: args.user_id,
            },
        )
        .await?;
    }

    Ok(())
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
