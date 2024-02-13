use std::collections::HashMap;

use anyhow::anyhow;
use async_recursion::async_recursion;
use sqlx::{Execute, QueryBuilder, Row, Sqlite};

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

pub struct FindMainTaskIdsArgs<'a> {
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}
// TODO: メインタスクは一つに制限することにしたので、VecではなくてStringを返すようにする
// また、これをSQLレベルで制限したいので、tasksにNULL許容のmain_task_idを追加する
pub async fn find_main_task_ids<'a>(
    db: &mut Connection,
    args: FindMainTaskIdsArgs<'a>,
) -> anyhow::Result<Vec<String>> {
    let main_ids = sqlx::query!(
        r#"
        SELECT id
        FROM sub_tasks sc 
            LEFT OUTER JOIN tasks t 
            ON (t.id = sc.main_task_id AND t.user_id = sc.user_id)
        WHERE sc.sub_task_id = $1 AND t.user_id = $2;
        "#,
        args.sub_task_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await?;

    let main_ids: Vec<String> = main_ids.into_iter().map(|r| r.id).collect();
    Ok(main_ids)
}

pub async fn update_all_unblocked_descendant_sub_tasks<'a>(
    db: &mut Connection,
    args: UpdateTaskStatusArgs<'a>,
) -> anyhow::Result<()> {
    let descendant_ids: Vec<String> = if args.status == &TaskStatus::Todo {
        // TODOに変更する場合は何もチェックしない
        let result = sqlx::query!(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT sub_task_id, main_task_id
                FROM sub_tasks
                WHERE main_task_id = $1 AND user_id = $2
    
                UNION
    
                SELECT s.sub_task_id, d.main_task_id
                FROM sub_tasks s
                JOIN descendants d ON s.main_task_id = d.sub_task_id
            )
    
            SELECT DISTINCT sub_task_id
            FROM descendants
            "#,
            args.id,
            args.user_id,
        )
        .fetch_all(&mut *db)
        .await?;

        result.into_iter().filter_map(|r| r.sub_task_id).collect()
    } else {
        // Doneに変更する場合は、ブロックされていないタスクだけを対象にする
        // ブロックされているタスクの子孫も除外したいんだけど、いい感じの方法が思いつかなかった。
        // とりあえずブロックされてるタスクを子孫も含めて全部取得して、そこに入ってないタスクを取得する
        let result = sqlx::query!(
            r#"
            WITH RECURSIVE all_blocked_tasks AS (
                SELECT blocking_task_id, blocked_task_id
                FROM blocking_tasks b
                JOIN tasks t ON (b.blocking_task_id = t.id)
                WHERE t.status = 'Todo'

                UNION

                SELECT blocking_task_id, sub_task_id as blocked_task_id
                FROM sub_tasks s
                JOIN all_blocked_tasks a ON (s.main_task_id = a.blocked_task_id)
            )
            , descendants AS (
                SELECT sub_task_id, main_task_id
                FROM sub_tasks
                WHERE main_task_id = $1 AND user_id = $2
    
                UNION
    
                SELECT s.sub_task_id, d.main_task_id
                FROM sub_tasks s
                JOIN descendants d ON s.main_task_id = d.sub_task_id
            )
    
            SELECT DISTINCT d.sub_task_id
            FROM descendants d
            LEFT OUTER JOIN all_blocked_tasks a ON (d.sub_task_id = a.blocked_task_id)
            WHERE a.blocked_task_id IS NULL
            "#,
            args.id,
            args.user_id,
        )
        .fetch_all(&mut *db)
        .await?;

        result.into_iter().filter_map(|r| r.sub_task_id).collect()
    };

    update_tasks_status(
        &mut *db,
        UpdateTasksStatusArgs {
            status: args.status,
            user_id: args.user_id,
            task_ids: &descendant_ids,
        },
    )
    .await?;

    Ok(())
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

pub struct TaskAndUser<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn update_all_main_tasks_status<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> anyhow::Result<()> {
    let main_ids = find_main_task_ids(
        &mut *db,
        FindMainTaskIdsArgs {
            sub_task_id: args.task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if main_ids.is_empty() {
        return Ok(());
    };

    update_tasks_and_all_ancestor_main_tasks_status(
        &mut *db,
        TasksAndUser {
            task_ids: &main_ids,
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
pub async fn update_tasks_and_all_ancestor_main_tasks_status<'a>(
    db: &mut Connection,
    args: TasksAndUser<'a>,
) -> anyhow::Result<()>
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

        // サブタスクから辿ったメインタスクがargs.tasksに入っているなら空にはならないが、サブタスクを持たないtaskで呼ばれる可能性がある
        if !task.sub_task_ids.is_empty() {
            let all_sub_tasks_done = is_all_tasks_done(&mut *db, &task.sub_task_ids).await?;
            let new_status = if all_sub_tasks_done {
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

        let main_ids = find_main_task_ids(
            &mut *db,
            FindMainTaskIdsArgs {
                sub_task_id: &task.id,
                user_id: args.user_id,
            },
        )
        .await?;

        // メインタスクのメインタスクも再帰的にに処理する
        update_tasks_and_all_ancestor_main_tasks_status(
            &mut *db,
            TasksAndUser {
                task_ids: &main_ids,
                user_id: args.user_id,
            },
        )
        .await?;
    }

    Ok(())
}

pub struct InsertSubTaskConnectionArgs<'a> {
    pub main_task_id: &'a str,
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn insert_sub_task_connection<'a>(
    db: &mut Connection,
    args: InsertSubTaskConnectionArgs<'a>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO sub_tasks(main_task_id, sub_task_id, user_id) VALUES($1, $2, $3) RETURNING *;",
        args.main_task_id,
        args.sub_task_id,
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
) -> anyhow::Result<()> {
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

pub enum SubTaskConnectionError {
    TaskNotFound,
    CircularTask,
    MultipleMainTask,
    BlockedByMainTask,
    Unknown(anyhow::Error),
}

pub async fn check_sub_task_connection<'a>(
    db: &mut Connection,
    args: &InsertSubTaskConnectionArgs<'a>,
) -> Result<(), SubTaskConnectionError> {
    // ログインユーザーが指定されたタスクを持っているかを確認する
    let tasks = sqlx::query!(
        "SELECT * FROM tasks WHERE id IN ($1, $2) AND user_id = $3;",
        args.main_task_id,
        args.sub_task_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await
    .map_err(|e| SubTaskConnectionError::Unknown(e.into()))?;

    if tasks.len() != 2 {
        return Err(SubTaskConnectionError::TaskNotFound);
    }

    // タスク同士が循環していないかを確認する。
    // payload.main_task_idの祖先に、payload.sub_task_idを持つtaskが存在しないことを確認する。
    if detect_circular_connection(
        &mut *db,
        DetectCircularConnectionArgs {
            parent_task_id: args.main_task_id,
            child_task_id: args.sub_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(SubTaskConnectionError::Unknown)?
    {
        return Err(SubTaskConnectionError::CircularTask);
    }

    // サブタスクがメインタスクにブロックされているタスクではないことを確認する
    if is_blocked_task(
        &mut *db,
        IsBlockedTaskArgs {
            blocking_task_id: args.main_task_id,
            task_id: args.sub_task_id,
        },
    )
    .await
    .map_err(SubTaskConnectionError::Unknown)?
    {
        return Err(SubTaskConnectionError::BlockedByMainTask);
    }

    // サブタスクが他のメインタスクを持っていないことを確認する
    if has_main_task(&mut *db, args.sub_task_id)
        .await
        .map_err(SubTaskConnectionError::Unknown)?
    {
        return Err(SubTaskConnectionError::MultipleMainTask);
    }

    Ok(())
}

pub enum BlockTaskConnectionError {
    TaskNotFound,
    IsSubTask,
    CircularTask,
    Unknown(anyhow::Error),
}

// TODO: sub_taskと共通化できる？
pub async fn check_insert_block_task_connection<'a>(
    db: &mut Connection,
    args: &InsertBlockTaskConnectionArgs<'a>,
) -> Result<(), BlockTaskConnectionError> {
    // ログインユーザーが指定されたタスクを持っているかを確認する
    let tasks = sqlx::query!(
        "SELECT * FROM tasks WHERE id IN ($1, $2) AND user_id = $3;",
        args.blocking_task_id,
        args.blocked_task_id,
        args.user_id,
    )
    .fetch_all(&mut *db)
    .await
    .map_err(|e| BlockTaskConnectionError::Unknown(e.into()))?;

    if tasks.len() != 2 {
        return Err(BlockTaskConnectionError::TaskNotFound);
    }

    if is_sub_task(
        &mut *db,
        IsSubTaskArgs {
            main_task_id: args.blocking_task_id,
            task_id: args.blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(BlockTaskConnectionError::Unknown)?
    {
        return Err(BlockTaskConnectionError::IsSubTask);
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
    .await
    .map_err(BlockTaskConnectionError::Unknown)?
    {
        return Err(BlockTaskConnectionError::CircularTask);
    }

    Ok(())
}

pub struct DeleteSubTaskConnectionArgs<'a> {
    pub main_task_id: &'a str,
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn delete_sub_task_connection<'a>(
    db: &mut Connection,
    args: DeleteSubTaskConnectionArgs<'a>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM sub_tasks WHERE main_task_id = $1 AND sub_task_id = $2 AND user_id = $3 RETURNING *",
        args.main_task_id,
        args.sub_task_id,
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
) -> anyhow::Result<()> {
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

pub struct IsSubTaskArgs<'a> {
    pub main_task_id: &'a str,
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn is_sub_task<'a>(db: &mut Connection, args: IsSubTaskArgs<'a>) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE all_sub_tasks AS (
            SELECT sub_task_id, main_task_id
            FROM sub_tasks
            WHERE main_task_id = $1 AND user_id = $2

            UNION

            SELECT sc.sub_task_id, s.main_task_id
            FROM sub_tasks sc
            JOIN all_sub_tasks s ON sc.main_task_id = s.sub_task_id
        )

        SELECT DISTINCT sub_task_id
        FROM all_sub_tasks
        WHERE sub_task_id = $3
        "#,
        args.main_task_id,
        args.user_id,
        args.task_id,
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}

pub async fn is_all_blocking_tasks_done(
    db: &mut Connection,
    task_id: &str,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors AS (
            -- 非再帰
            -- サブタスクのメインタスクのステータスは関係ないのでNULLにする
            SELECT main_task_id, sub_task_id as child_task_id, NULL as parent_task_status
            FROM sub_tasks
            WHERE sub_task_id = $1

            UNION

            -- ブロックしているタスクのステータスをSELECTする
            SELECT blocking_task_id as main_task_id, blocked_task_id as child_task_id, status
            FROM blocking_tasks b JOIN tasks t ON b.blocking_task_id = t.id
            WHERE blocked_task_id = $1

            UNION

            -- 再帰
            SELECT s.main_task_id, a.child_task_id, NULL as status
            FROM sub_tasks s
            JOIN ancestors a ON s.sub_task_id = a.main_task_id

            UNION

            SELECT b.blocking_task_id, a.child_task_id, t.status
            FROM blocking_tasks b
            JOIN ancestors a ON b.blocked_task_id = a.main_task_id
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

pub async fn has_main_task(db: &mut Connection, task_id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query!("SELECT * FROM sub_tasks WHERE sub_task_id = $1", task_id)
        .fetch_all(&mut *db)
        .await?;

    Ok(!result.is_empty())
}

pub struct IsBlockedTaskArgs<'a> {
    pub blocking_task_id: &'a str,
    pub task_id: &'a str,
}
pub async fn is_blocked_task<'a>(
    db: &mut Connection,
    args: IsBlockedTaskArgs<'a>,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        "SELECT * FROM blocking_tasks WHERE blocking_task_id = $1 AND blocked_task_id = $2;",
        args.blocking_task_id,
        args.task_id
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(!result.is_empty())
}
