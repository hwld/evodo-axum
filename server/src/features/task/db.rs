use anyhow::anyhow;
use sqlx::{Execute, QueryBuilder, Row, Sqlite};
use std::collections::HashMap;

use crate::app::Connection;

use super::{Task, TaskStatus};

pub struct FindTaskArgs<'a> {
    pub user_id: &'a str,
    pub task_id: &'a str,
}
pub async fn find_task<'a>(
    db: &mut Connection,
    FindTaskArgs { user_id, task_id }: FindTaskArgs<'a>,
) -> anyhow::Result<Task> {
    let raw_task = sqlx::query!(
        r#"
        SELECT t.*, s.main_task_id, s.sub_task_id, b.blocked_task_id
        FROM tasks t 
        LEFT OUTER JOIN sub_tasks s ON (t.id = s.main_task_id AND t.user_id = s.user_id)
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
            sub_task_ids: Vec::new(),
            blocked_task_ids: Vec::new(),
        });
        if let Some(sub_task_id) = raw.sub_task_id {
            task.sub_task_ids.push(sub_task_id);
        }
        if let Some(blocked_task_id) = raw.blocked_task_id {
            task.blocked_task_ids.push(blocked_task_id);
        }
    }
    let task = task_map
        .into_iter()
        .next()
        .map(|(_, mut t)| {
            t.sub_task_ids.sort();
            t.sub_task_ids.dedup();
            t.blocked_task_ids.sort();
            t.blocked_task_ids.dedup();
            t
        })
        .ok_or(anyhow!("Error"))?;

    Ok(task)
}

pub async fn find_tasks(db: &mut Connection, user_id: &str) -> anyhow::Result<Vec<Task>> {
    let raw_tasks = sqlx::query!(
        r#"
        SELECT t.*, s.main_task_id, s.sub_task_id, b.blocked_task_id
        FROM tasks t 
        LEFT OUTER JOIN sub_tasks s ON (t.id = s.main_task_id AND t.user_id = s.user_id)
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
            sub_task_ids: Vec::new(),
            blocked_task_ids: Vec::new(),
        });
        if let Some(sub_task_id) = raw.sub_task_id {
            task.sub_task_ids.push(sub_task_id);
        }
        if let Some(blocked_task_id) = raw.blocked_task_id {
            task.blocked_task_ids.push(blocked_task_id);
        }
    }
    let tasks: Vec<Task> = task_map
        .into_values()
        .map(|mut t| {
            t.sub_task_ids.sort();
            t.sub_task_ids.dedup();
            t.blocked_task_ids.sort();
            t.blocked_task_ids.dedup();
            t
        })
        .collect();

    Ok(tasks)
}

pub async fn is_all_tasks_done(
    db: &mut Connection,
    task_ids: &Vec<String>,
) -> anyhow::Result<bool> {
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
pub async fn insert_task<'a>(
    db: &mut Connection,
    args: InsertTaskArgs<'a>,
) -> anyhow::Result<Task> {
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
pub async fn delete_task<'a>(
    db: &mut Connection,
    args: DeleteTaskArgs<'a>,
) -> anyhow::Result<String> {
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
pub async fn update_task<'a>(
    db: &mut Connection,
    args: UpdateTaskArgs<'a>,
) -> anyhow::Result<Task> {
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
) -> anyhow::Result<Task> {
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
) -> anyhow::Result<()> {
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

pub struct DetectCircularConnectionArgs<'a> {
    pub parent_task_id: &'a str,
    pub child_task_id: &'a str,
    pub user_id: &'a str,
}

/// タスク同士が循環接続になりうるかを判定する
pub async fn detect_circular_connection<'a>(
    db: &mut Connection,
    args: DetectCircularConnectionArgs<'a>,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            -- 非再帰
            SELECT sub_task_id as child_id, main_task_id as parent_id
            FROM sub_tasks
            WHERE sub_task_id = $1 AND user_id = $2

            UNION

            SELECT blocked_task_id as child_id, blocking_task_id as parent_id
            FROM blocking_tasks
            WHERE blocked_task_id = $1 AND user_id = $2

            UNION
            -- 再帰
            SELECT s.sub_task_id, s.main_task_id
            FROM sub_tasks s
            JOIN ancestors a ON s.sub_task_id = a.parent_id

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
