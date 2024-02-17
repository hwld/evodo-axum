use crate::{app::Connection, features::block_task::db::BlockTaskConnectionError};

use super::{
    connect_block_task::{self, ConnectBlockTaskArgs, ConnectBlockTaskError},
    disconnect_block_task::{self, DisconnectBlockTaskArgs},
};

pub struct ReconnectBlockTaskArgs<'a> {
    pub old_blocking_task_id: &'a str,
    pub old_blocked_task_id: &'a str,
    pub new_blocking_task_id: &'a str,
    pub new_blocked_task_id: &'a str,
    pub user_id: &'a str,
}

pub enum ReconnectBlockTaskError {
    Connect(BlockTaskConnectionError),
    Unknown(anyhow::Error),
}

pub async fn action<'a>(
    db: &mut Connection,
    args: ReconnectBlockTaskArgs<'a>,
) -> Result<(), ReconnectBlockTaskError> {
    disconnect_block_task::action(
        db,
        DisconnectBlockTaskArgs {
            blocking_task_id: args.old_blocking_task_id,
            blocked_task_id: args.old_blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(ReconnectBlockTaskError::Unknown)?;

    connect_block_task::action(
        db,
        ConnectBlockTaskArgs {
            blocking_task_id: args.new_blocking_task_id,
            blocked_task_id: args.new_blocked_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(|e| {
        use ReconnectBlockTaskError::{Connect, Unknown};

        match e {
            ConnectBlockTaskError::CheckError(err) => Connect(err),
            ConnectBlockTaskError::Unknown(err) => Unknown(err),
        }
    })?;

    Ok(())
}
