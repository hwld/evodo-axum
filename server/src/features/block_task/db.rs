use crate::{
    app::Connection,
    features::{
        sub_task::db::{is_sub_task, IsSubTaskArgs},
        task::{
            db::{
                detect_circular_connection, exists_tasks, update_tasks_status,
                DetectCircularConnectionArgs, ExistsTasksArg, ExistsTasksError,
                UpdateTaskStatusArgs, UpdateTasksStatusArgs,
            },
            TaskStatus,
        },
    },
};

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

pub enum BlockTaskConnectionError {
    TaskNotFound,
    IsSubTask,
    CircularTask,
    Unknown(anyhow::Error),
}

pub async fn check_insert_block_task_connection<'a>(
    db: &mut Connection,
    args: &InsertBlockTaskConnectionArgs<'a>,
) -> Result<(), BlockTaskConnectionError> {
    exists_tasks(
        &mut *db,
        ExistsTasksArg {
            task_ids: &vec![args.blocking_task_id, args.blocked_task_id],
            user_id: args.user_id,
        },
    )
    .await
    .map_err(|e| {
        use ExistsTasksError::{TasksNotFound, Unknown};

        match e {
            TasksNotFound => BlockTaskConnectionError::TaskNotFound,
            Unknown(err) => BlockTaskConnectionError::Unknown(err.into()),
        }
    })?;

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
