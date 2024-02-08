use crate::{
    app::{AppResult, Connection},
    features::task::db::{delete_block_task_connection, DeleteBlockTaskConnectionArgs},
};

pub struct DisconnectBlockTaskArgs<'a> {
    pub blocking_task_id: &'a str,
    pub blocked_task_id: &'a str,
    pub user_id: &'a str,
}
pub async fn action<'a>(db: &mut Connection, args: DisconnectBlockTaskArgs<'a>) -> AppResult<()> {
    delete_block_task_connection(
        db,
        DeleteBlockTaskConnectionArgs {
            blocking_task_id: args.blocking_task_id,
            blocked_task_id: args.blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await?;

    Ok(())
}
