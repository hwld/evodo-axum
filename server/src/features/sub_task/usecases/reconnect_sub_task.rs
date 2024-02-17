use crate::{
    app::Connection,
    features::{
        sub_task::usecases::connect_sub_task::ConnectSubTaskError, task::db::SubTaskConnectionError,
    },
};

use super::{
    connect_sub_task::{self, ConnectSubTaskArgs},
    disconnect_sub_task::{self, DisconnectSubTaskArgs},
};

pub struct ReconnectSubTaskArgs<'a> {
    pub old_main_task_id: &'a str,
    pub old_sub_task_id: &'a str,
    pub new_main_task_id: &'a str,
    pub new_sub_task_id: &'a str,
    pub user_id: &'a str,
}

pub enum ReconnectSubTaskError {
    Connect(SubTaskConnectionError),
    Unknown(anyhow::Error),
}

pub async fn action<'a>(
    db: &mut Connection,
    args: ReconnectSubTaskArgs<'a>,
) -> Result<(), ReconnectSubTaskError> {
    disconnect_sub_task::action(
        db,
        DisconnectSubTaskArgs {
            main_task_id: args.old_main_task_id,
            sub_task_id: args.old_sub_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(ReconnectSubTaskError::Unknown)?;

    connect_sub_task::action(
        db,
        ConnectSubTaskArgs {
            main_task_id: args.new_main_task_id,
            sub_task_id: args.new_sub_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(|e| {
        use ReconnectSubTaskError::{Connect, Unknown};
        match e {
            ConnectSubTaskError::CheckError(err) => Connect(err),
            ConnectSubTaskError::Unknown(err) => Unknown(err),
        }
    })?;

    Ok(())
}
