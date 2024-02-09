use std::collections::HashMap;

use anyhow::anyhow;
use async_recursion::async_recursion;
use sqlx::{Execute, QueryBuilder, Row, Sqlite};

use crate::app::{AppResult, Connection};

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
        SELECT t.*, s.parent_task_id, s.subtask_id, b.blocked_task_id
        FROM tasks t 
        LEFT OUTER JOIN subtask_connections s ON (t.id = s.parent_task_id AND t.user_id = s.user_id)
        LEFT OUTER JOIN blocking_tasks b ON (t.id = b.blocking_task_id AND t.user_id = b.user_id)
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
            description: raw.description,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            subtask_ids: Vec::new(),
            blocked_task_ids: Vec::new(),
        });
        if let Some(subtask_id) = raw.subtask_id {
            task.subtask_ids.push(subtask_id);
        }
        if let Some(blocked_task_id) = raw.blocked_task_id {
            task.blocked_task_ids.push(blocked_task_id);
        }
    }
    let task = task_map
        .into_iter()
        .next()
        .map(|(_, mut t)| {
            t.subtask_ids.sort();
            t.subtask_ids.dedup();
            t.blocked_task_ids.sort();
            t.blocked_task_ids.dedup();
            t
        })
        .ok_or(anyhow!("Error"))?;

    Ok(task)
}

pub async fn find_tasks(db: &mut Connection, user_id: &str) -> AppResult<Vec<Task>> {
    let raw_tasks = sqlx::query!(
        r#"
        SELECT t.*, s.parent_task_id, s.subtask_id, b.blocked_task_id
        FROM tasks t 
        LEFT OUTER JOIN subtask_connections s ON (t.id = s.parent_task_id AND t.user_id = s.user_id)
        LEFT OUTER JOIN blocking_tasks b ON (t.id = b.blocking_task_id AND t.user_id = b.user_id)
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
            description: raw.description,
            user_id: raw.user_id,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            subtask_ids: Vec::new(),
            blocked_task_ids: Vec::new(),
        });
        if let Some(subtask_id) = raw.subtask_id {
            task.subtask_ids.push(subtask_id);
        }
        if let Some(blocked_task_id) = raw.blocked_task_id {
            task.blocked_task_ids.push(blocked_task_id);
        }
    }
    let tasks: Vec<Task> = task_map
        .into_values()
        .map(|mut t| {
            t.subtask_ids.sort();
            t.subtask_ids.dedup();
            t.blocked_task_ids.sort();
            t.blocked_task_ids.dedup();
            t
        })
        .collect();

    Ok(tasks)
}

pub struct FindParentTaskIdsArgs<'a> {
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn find_parent_task_ids<'a>(
    db: &mut Connection,
    args: FindParentTaskIdsArgs<'a>,
) -> AppResult<Vec<String>> {
    let parent_ids = sqlx::query!(
        r#"
        SELECT id
        FROM subtask_connections sc 
            LEFT OUTER JOIN tasks t 
            ON (t.id = sc.parent_task_id AND t.user_id = sc.user_id)
        WHERE sc.subtask_id = $1 AND t.user_id = $2;
        "#,
        args.subtask_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    let parent_ids: Vec<String> = parent_ids.into_iter().map(|r| r.id).collect();
    Ok(parent_ids)
}

pub async fn find_all_descendant_task_ids<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> AppResult<Vec<String>> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE descendants AS (
            SELECT subtask_id, parent_task_id
            FROM subtask_connections
            WHERE parent_task_id = $1 AND user_id = $2

            UNION

            SELECT s.subtask_id, d.parent_task_id
            FROM subtask_connections s
            JOIN descendants d ON s.parent_task_id = d.subtask_id
        )

        SELECT DISTINCT subtask_id
        FROM descendants
        "#,
        args.task_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    let ids: Vec<String> = result.into_iter().filter_map(|r| r.subtask_id).collect();
    Ok(ids)
}

pub async fn is_all_tasks_done(db: &mut Connection, task_ids: &Vec<String>) -> AppResult<bool> {
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
    pub description: &'a str,
    pub user_id: &'a str,
    pub status: &'a TaskStatus,
}
pub async fn insert_task<'a>(db: &mut Connection, args: InsertTaskArgs<'a>) -> AppResult<Task> {
    let result = sqlx::query!(
        r#" INSERT INTO tasks(id, title, description, user_id, status) VALUES($1, $2, $3, $4, $5) RETURNING *"#,
        args.id,
        args.title,
        args.description,
        args.user_id,
        args.status
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

pub struct DeleteTaskArgs<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}
pub async fn delete_task<'a>(db: &mut Connection, args: DeleteTaskArgs<'a>) -> AppResult<String> {
    let result = sqlx::query!(
        r#"DELETE FROM tasks WHERE id = $1 AND user_id = $2 RETURNING *;"#,
        args.id,
        args.user_id
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(result.id)
}

pub struct UpdateTaskArgs<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub user_id: &'a str,
}
pub async fn update_task<'a>(db: &mut Connection, args: UpdateTaskArgs<'a>) -> AppResult<Task> {
    let result = sqlx::query!(
        r#"
        UPDATE
            tasks 
        SET
            title = $1,
            description = $2
        WHERE
            id = $3 AND user_id = $4
        RETURNING *;        
        "#,
        args.title,
        args.description,
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
            status = $1
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

pub struct UpdateTasksStatusArgs<'a> {
    pub task_ids: &'a Vec<String>,
    pub status: &'a TaskStatus,
    pub user_id: &'a str,
}
pub async fn update_tasks_status<'a>(
    db: &mut Connection,
    args: UpdateTasksStatusArgs<'a>,
) -> AppResult<()> {
    if args.task_ids.is_empty() {
        return Ok(());
    };

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
        UPDATE
            tasks 
        SET
            status = "#,
    );
    let query_builder = query_builder.push_bind(args.status);
    let query_builder = query_builder.push(
        r#"
        WHERE
            id IN ("#,
    );

    let mut separated = query_builder.separated(", ");
    for id in args.task_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(") AND user_id = ");
    query_builder.push_bind(args.user_id);

    let query = query_builder.build();
    println!("{}", query.sql());
    query.execute(&mut *db).await?;

    Ok(())
}

pub struct TaskAndUser<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn update_all_ancestors_task_status<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> AppResult<()> {
    let parent_ids = find_parent_task_ids(
        &mut *db,
        FindParentTaskIdsArgs {
            subtask_id: args.task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if parent_ids.is_empty() {
        return Ok(());
    };

    update_tasks_and_ancestors_status(
        &mut *db,
        TasksAndUser {
            task_ids: &parent_ids,
            user_id: args.user_id,
        },
    )
    .await?;

    Ok(())
}

pub struct TasksAndUser<'a> {
    pub task_ids: &'a Vec<String>,
    pub user_id: &'a str,
}

// パフォーマンス悪いかも
#[async_recursion]
pub async fn update_tasks_and_ancestors_status<'a>(
    db: &mut Connection,
    args: TasksAndUser<'a>,
) -> AppResult<()>
where
    'a: 'async_recursion,
{
    if args.task_ids.is_empty() {
        return Ok(());
    }

    for task_id in args.task_ids {
        let task = find_task(
            &mut *db,
            FindTaskArgs {
                task_id,
                user_id: args.user_id,
            },
        )
        .await?;

        // 子から辿った親がargs.tasksに入っているなら空にはならないが、子を持たないtaskで呼ばれる可能性がある
        if !task.subtask_ids.is_empty() {
            let all_subtasks_done = is_all_tasks_done(&mut *db, &task.subtask_ids).await?;
            let new_status = if all_subtasks_done {
                TaskStatus::Done
            } else {
                TaskStatus::Todo
            };

            // 渡されたタスクが多いことを想定するならupdateをまとめたほうがいいかもしれないけど、多くならないと思う
            update_task_status(
                &mut *db,
                UpdateTaskStatusArgs {
                    id: &task.id,
                    status: &new_status,
                    user_id: args.user_id,
                },
            )
            .await?;
        }

        let parent_ids = find_parent_task_ids(
            &mut *db,
            FindParentTaskIdsArgs {
                subtask_id: &task.id,
                user_id: args.user_id,
            },
        )
        .await?;

        // 親の祖先も同じように処理する
        update_tasks_and_ancestors_status(
            &mut *db,
            TasksAndUser {
                task_ids: &parent_ids,
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

pub struct InsertBlockTaskConnectionArgs<'a> {
    pub blocking_task_id: &'a str,
    pub blocked_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn insert_block_task_connection<'a>(
    db: &mut Connection,
    args: InsertBlockTaskConnectionArgs<'a>,
) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO blocking_tasks(blocking_task_id, blocked_task_id, user_id) VALUES ($1, $2, $3) RETURNING *;",
        args.blocking_task_id,
        args.blocked_task_id,
        args.user_id
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(())
}

pub async fn check_subtask_connection<'a>(
    db: &mut Connection,
    args: &InsertSubTaskConnectionArgs<'a>,
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
        return Err(anyhow!("タスクが存在しません").into());
    }

    // タスク同士が循環していないかを確認する。
    // payload.parent_task_idの祖先に、payload.subtask_idを持つtaskが存在しないことを確認する。
    if detect_circular_connection(
        &mut *db,
        DetectCircularConnectionArgs {
            parent_task_id: args.parent_task_id,
            child_task_id: args.subtask_id,
            user_id: args.user_id,
        },
    )
    .await?
    {
        return Err(anyhow!("タスクの循環は許可されていません").into());
    }

    // 子孫タスクではないことを確認する
    if is_descendant_task(
        &mut *db,
        IsDescendantTaskArgs {
            parent_task_id: args.parent_task_id,
            task_id: args.subtask_id,
        },
    )
    .await?
    {
        return Err(anyhow!("子孫タスクをサブタスクにすることはできません").into());
    }

    Ok(())
}

// TODO: subtaskと共通化できる？
pub async fn check_insert_block_task_connection<'a>(
    db: &mut Connection,
    args: &InsertBlockTaskConnectionArgs<'a>,
) -> AppResult<()> {
    // ログインユーザーが指定されたタスクを持っているかを確認する
    let tasks = sqlx::query!(
        "SELECT * FROM tasks WHERE id IN ($1, $2) AND user_id = $3;",
        args.blocking_task_id,
        args.blocked_task_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    if tasks.len() != 2 {
        return Err(anyhow!("タスクが存在しません").into());
    }

    if is_subtask(
        &mut *db,
        IsSubtaskArgs {
            parent_task_id: args.blocking_task_id,
            task_id: args.blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await?
    {
        return Err(anyhow!("サブタスクをブロックすることはできません").into());
    };

    // タスク同士が循環していないかを確認する。
    if detect_circular_connection(
        &mut *db,
        DetectCircularConnectionArgs {
            parent_task_id: args.blocking_task_id,
            child_task_id: args.blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await?
    {
        return Err(anyhow!("タスクの循環は許可されていません").into());
    }

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

pub struct DeleteBlockTaskConnectionArgs<'a> {
    pub blocking_task_id: &'a str,
    pub blocked_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn delete_block_task_connection<'a>(
    db: &mut Connection,
    args: DeleteBlockTaskConnectionArgs<'a>,
) -> AppResult<()> {
    sqlx::query!(
        "DELETE FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2 AND user_id = $3 RETURNING *",
        args.blocking_task_id,
        args.blocked_task_id,
        args.user_id
    ).fetch_one(&mut *db).await?;

    Ok(())
}

pub struct DetectCircularConnectionArgs<'a> {
    parent_task_id: &'a str,
    child_task_id: &'a str,
    user_id: &'a str,
}

/// タスク同士が循環接続になりうるかを判定する
pub async fn detect_circular_connection<'a>(
    db: &mut Connection,
    args: DetectCircularConnectionArgs<'a>,
) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            -- 非再帰
            SELECT subtask_id as child_id, parent_task_id as parent_id
            FROM subtask_connections
            WHERE subtask_id = $1 AND user_id = $2

            UNION

            SELECT blocked_task_id as child_id, blocking_task_id as parent_id
            FROM blocking_tasks
            WHERE blocked_task_id = $1 AND user_id = $2

            UNION
            -- 再帰
            SELECT s.subtask_id, s.parent_task_id
            FROM subtask_connections s
            JOIN ancestors a ON s.subtask_id = a.parent_id

            UNION

            SELECT b.blocked_task_id, b.blocking_task_id
            FROM blocking_tasks b
            JOIN ancestors a ON b.blocked_task_id = a.parent_id
        )

        SELECT DISTINCT parent_id
        FROM ancestors
        WHERE parent_id = $3
        "#,
        args.parent_task_id,
        args.user_id,
        args.child_task_id,
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}

pub struct IsSubtaskArgs<'a> {
    pub parent_task_id: &'a str,
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn is_subtask<'a>(db: &mut Connection, args: IsSubtaskArgs<'a>) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE subtasks AS (
            SELECT subtask_id, parent_task_id
            FROM subtask_connections
            WHERE parent_task_id = $1 AND user_id = $2

            UNION

            SELECT sc.subtask_id, s.parent_task_id
            FROM subtask_connections sc
            JOIN subtasks s ON sc.parent_task_id = s.subtask_id
        )

        SELECT DISTINCT subtask_id
        FROM subtasks
        WHERE subtask_id = $3
        "#,
        args.parent_task_id,
        args.user_id,
        args.task_id,
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}

pub async fn is_all_blocking_tasks_done(db: &mut Connection, task_id: &str) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            -- 非再帰
            -- サブタスクの親のステータスは関係ないのでNULLにする
            SELECT parent_task_id, subtask_id as child_task_id, NULL as parent_task_status
            FROM subtask_connections
            WHERE subtask_id = $1

            UNION

            SELECT blocking_task_id as parent_task_id, blocked_task_id as child_task_id, status
            FROM blocking_tasks b JOIN tasks t ON b.blocking_task_id = t.id
            WHERE blocked_task_id = $1

            UNION

            -- 再帰
            SELECT s.parent_task_id, a.child_task_id, NULL as status
            FROM subtask_connections s
            JOIN ancestors a ON s.subtask_id = a.parent_task_id

            UNION

            SELECT b.blocking_task_id, a.child_task_id, t.status
            FROM blocking_tasks b
            JOIN ancestors a ON b.blocked_task_id = a.parent_task_id
            JOIN tasks t ON b.blocking_task_id = t.id
        )

        SELECT COUNT(*) as non_done_status_count
        FROM ancestors a
        WHERE a.parent_task_status != 'Done'
        "#,
        task_id
    )
    .fetch_one(db)
    .await?;

    Ok(result.non_done_status_count == 0)
}

pub struct IsDescendantTaskArgs<'a> {
    pub parent_task_id: &'a str,
    pub task_id: &'a str,
}
pub async fn is_descendant_task<'a>(
    db: &mut Connection,
    args: IsDescendantTaskArgs<'a>,
) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE descendants AS (
            -- 非再帰
            SELECT parent_task_id, subtask_id as child_task_id
            FROM subtask_connections
            WHERE parent_task_id = $1

            UNION

            SELECT blocking_task_id as parent_task_id, blocked_task_id as child_task_id
            FROM blocking_tasks
            WHERE blocking_task_id = $1

            UNION

            -- 再帰
            SELECT d.parent_task_id, s.subtask_id as child_task_id
            FROM subtask_connections s
            JOIN descendants d ON s.parent_task_id = d.child_task_id

            UNION

            SELECT d.parent_task_id, b.blocked_task_id as child_task_id
            FROM blocking_tasks b
            JOIN descendants d ON b.blocking_task_id = d.child_task_id
        )

        SELECT *
        FROM descendants
        WHERE child_task_id = $2
        "#,
        args.parent_task_id,
        args.task_id
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}
