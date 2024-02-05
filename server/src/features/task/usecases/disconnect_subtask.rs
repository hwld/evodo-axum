use crate::{
    app::{AppResult, Connection},
    features::task::db::{
        delete_subtask_connection, find_parent_task_ids, update_tasks_and_ancestors_status,
        DeleteSubTaskConnectionArgs, FindParentTaskIdsArgs, TasksAndUser,
    },
};

pub struct DisconnectSubtaskArgs<'a> {
    pub parent_task_id: &'a str,
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn action<'a>(db: &mut Connection, args: DisconnectSubtaskArgs<'a>) -> AppResult<()> {
    // 後でサブタスクの親すべてを更新する必要があるので、subtask_connectionを削除する前に親を取得しておく
    let parent_ids = find_parent_task_ids(
        &mut *db,
        FindParentTaskIdsArgs {
            subtask_id: args.subtask_id,
            user_id: args.user_id,
        },
    )
    .await?;

    delete_subtask_connection(
        db,
        DeleteSubTaskConnectionArgs {
            parent_task_id: args.parent_task_id,
            subtask_id: args.subtask_id,
            user_id: args.user_id,
        },
    )
    .await?;

    // 接続を切り離したサブタスクの祖先の状態をすべて更新する
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
