use crate::{
    app::Connection,
    features::sub_task::db::{
        delete_sub_task_connection, find_main_task_id,
        update_task_and_all_ancestor_main_tasks_status, DeleteSubTaskConnectionArgs,
        FindMainTaskIdsArgs, TaskAndUser,
    },
};

pub struct DisconnectSubTaskArgs<'a> {
    pub main_task_id: &'a str,
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn action<'a>(
    db: &mut Connection,
    args: DisconnectSubTaskArgs<'a>,
) -> anyhow::Result<()> {
    // 後でサブタスクのメインタスクをすべてを更新する必要があるので、sub_tasksを削除する前に直近のメインタスクを取得しておく
    let main_task_id = find_main_task_id(
        &mut *db,
        FindMainTaskIdsArgs {
            sub_task_id: args.sub_task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    delete_sub_task_connection(
        db,
        DeleteSubTaskConnectionArgs {
            main_task_id: args.main_task_id,
            sub_task_id: args.sub_task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    if let Some(id) = main_task_id {
        // 接続を切り離したサブタスクのすべての祖先メインタスクの状態を更新する
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
