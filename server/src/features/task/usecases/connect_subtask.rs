use crate::{
    app::Connection,
    features::task::db::{
        check_subtask_connection, insert_subtask_connection, update_all_ancestors_task_status,
        InsertSubTaskConnectionArgs, SubtaskConnectionError, TaskAndUser,
    },
};

pub struct ConnectSubtaskArgs<'a> {
    pub parent_task_id: &'a str,
    pub subtask_id: &'a str,
    pub user_id: &'a str,
}

pub enum ConnectSubtaskError {
    CheckError(SubtaskConnectionError),
    Unknown(anyhow::Error),
}

pub async fn action<'a>(
    db: &mut Connection,
    args: ConnectSubtaskArgs<'a>,
) -> Result<(), ConnectSubtaskError> {
    let insert_args = InsertSubTaskConnectionArgs {
        parent_task_id: args.parent_task_id,
        subtask_id: args.subtask_id,
        user_id: args.user_id,
    };

    check_subtask_connection(db, &insert_args)
        .await
        .map_err(ConnectSubtaskError::CheckError)?;

    insert_subtask_connection(db, insert_args)
        .await
        .map_err(ConnectSubtaskError::Unknown)?;

    update_all_ancestors_task_status(
        db,
        TaskAndUser {
            task_id: args.subtask_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(ConnectSubtaskError::Unknown)?;

    Ok(())
}
