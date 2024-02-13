use anyhow::anyhow;

use crate::app::{AppResult, Connection};

use super::{
    connect_subtask::{self, ConnectSubtaskArgs},
    disconnect_subtask::{self, DisconnectSubtaskArgs},
};

pub struct ReconnectSubtaskArgs<'a> {
    pub old_parent_task_id: &'a str,
    pub old_subtask_id: &'a str,
    pub new_parent_task_id: &'a str,
    pub new_subtask_id: &'a str,
    pub user_id: &'a str,
}
pub async fn action<'a>(db: &mut Connection, args: ReconnectSubtaskArgs<'a>) -> AppResult<()> {
    disconnect_subtask::action(
        db,
        DisconnectSubtaskArgs {
            parent_task_id: args.old_parent_task_id,
            subtask_id: args.old_subtask_id,
            user_id: args.user_id,
        },
    )
    .await?;

    // TODO
    connect_subtask::action(
        db,
        ConnectSubtaskArgs {
            parent_task_id: args.new_parent_task_id,
            subtask_id: args.new_subtask_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(|_| anyhow!("error"))?;

    Ok(())
}
