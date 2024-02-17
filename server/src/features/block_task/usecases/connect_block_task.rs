use crate::{
    app::Connection,
    features::task::db::{
        check_insert_block_task_connection, insert_block_task_connection, BlockTaskConnectionError,
        InsertBlockTaskConnectionArgs,
    },
};

pub struct ConnectBlockTaskArgs<'a> {
    pub blocking_task_id: &'a str,
    pub blocked_task_id: &'a str,
    pub user_id: &'a str,
}

pub enum ConnectBlockTaskError {
    CheckError(BlockTaskConnectionError),
    Unknown(anyhow::Error),
}

pub async fn action<'a>(
    db: &mut Connection,
    args: ConnectBlockTaskArgs<'a>,
) -> Result<(), ConnectBlockTaskError> {
    let insert_args = InsertBlockTaskConnectionArgs {
        blocking_task_id: args.blocking_task_id,
        blocked_task_id: args.blocked_task_id,
        user_id: args.user_id,
    };

    check_insert_block_task_connection(db, &insert_args)
        .await
        .map_err(ConnectBlockTaskError::CheckError)?;

    insert_block_task_connection(db, insert_args)
        .await
        .map_err(ConnectBlockTaskError::Unknown)?;

    Ok(())
}
