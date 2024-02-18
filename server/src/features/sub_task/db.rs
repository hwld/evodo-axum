use async_recursion::async_recursion;

use crate::{
    app::Connection,
    features::{
        block_task::db::{is_blocked_task, IsBlockedTaskArgs},
        task::{
            db::{
                detect_circular_connection, exists_tasks, find_task, is_all_tasks_done,
                update_task_status, DetectCircularConnectionArgs, ExistsTasksArg, ExistsTasksError,
                FindTaskArgs, UpdateTaskStatusArgs,
            },
            TaskStatus,
        },
    },
};

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
    exists_tasks(
        &mut *db,
        ExistsTasksArg {
            task_ids: &vec![args.main_task_id, args.sub_task_id],
            user_id: args.user_id,
        },
    )
    .await
    .map_err(|e| {
        use ExistsTasksError::*;

        match e {
            TasksNotFound => SubTaskConnectionError::TaskNotFound,
            Unknown(e) => SubTaskConnectionError::Unknown(e.into()),
        }
    })?;

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

pub async fn has_main_task(db: &mut Connection, task_id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query!("SELECT * FROM sub_tasks WHERE sub_task_id = $1", task_id)
        .fetch_all(&mut *db)
        .await?;

    Ok(!result.is_empty())
}

pub struct FindMainTaskIdsArgs<'a> {
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}

pub async fn find_main_task_id<'a>(
    db: &mut Connection,
    args: FindMainTaskIdsArgs<'a>,
) -> anyhow::Result<Option<String>> {
    let result = sqlx::query!(
        r#"SELECT main_task_id FROM sub_tasks WHERE sub_task_id = $1 AND user_id = $2;"#,
        args.sub_task_id,
        args.user_id,
    )
    .fetch_optional(&mut *db)
    .await?;

    Ok(result.map(|r| r.main_task_id))
}

pub struct TaskAndUser<'a> {
    pub task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn update_all_ancestor_main_tasks_status<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> anyhow::Result<()> {
    let main_task_id = find_main_task_id(
        &mut *db,
        FindMainTaskIdsArgs {
            sub_task_id: args.task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if let Some(id) = main_task_id {
        update_task_and_all_ancestor_main_tasks_status(
            &mut *db,
            TaskAndUser {
                task_id: &id,
                user_id: args.user_id,
            },
        )
        .await?;
    }

    Ok(())
}

// パフォーマンス悪いかも
#[async_recursion]
pub async fn update_task_and_all_ancestor_main_tasks_status<'a>(
    db: &mut Connection,
    args: TaskAndUser<'a>,
) -> anyhow::Result<()>
where
    'a: 'async_recursion,
{
    let task = find_task(
        &mut *db,
        FindTaskArgs {
            task_id: args.task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if !task.sub_task_ids.is_empty() {
        // サブタスクの状態を見てタスクの状態を更新する
        let is_all_sub_tasks_done = is_all_tasks_done(&mut *db, &task.sub_task_ids).await?;
        let new_status = if is_all_sub_tasks_done {
            TaskStatus::Done
        } else {
            TaskStatus::Todo
        };

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

    let main_task_id = find_main_task_id(
        &mut *db,
        FindMainTaskIdsArgs {
            sub_task_id: &task.id,
            user_id: args.user_id,
        },
    )
    .await?;

    if let Some(id) = main_task_id {
        update_task_and_all_ancestor_main_tasks_status(
            &mut *db,
            TaskAndUser {
                task_id: &id,
                user_id: args.user_id,
            },
        )
        .await?;
    }

    Ok(())
}
