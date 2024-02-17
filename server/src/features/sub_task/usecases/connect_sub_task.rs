use crate::{
    app::Connection,
    features::task::db::{
        check_sub_task_connection, insert_sub_task_connection,
        update_all_ancestor_main_tasks_status, InsertSubTaskConnectionArgs, SubTaskConnectionError,
        TaskAndUser,
    },
};

pub struct ConnectSubTaskArgs<'a> {
    pub main_task_id: &'a str,
    pub sub_task_id: &'a str,
    pub user_id: &'a str,
}

pub enum ConnectSubTaskError {
    CheckError(SubTaskConnectionError),
    Unknown(anyhow::Error),
}

pub async fn action<'a>(
    db: &mut Connection,
    args: ConnectSubTaskArgs<'a>,
) -> Result<(), ConnectSubTaskError> {
    let insert_args = InsertSubTaskConnectionArgs {
        main_task_id: args.main_task_id,
        sub_task_id: args.sub_task_id,
        user_id: args.user_id,
    };

    check_sub_task_connection(db, &insert_args)
        .await
        .map_err(ConnectSubTaskError::CheckError)?;

    insert_sub_task_connection(db, insert_args)
        .await
        .map_err(ConnectSubTaskError::Unknown)?;

    update_all_ancestor_main_tasks_status(
        db,
        TaskAndUser {
            task_id: args.sub_task_id,
            user_id: args.user_id,
        },
    )
    .await
    .map_err(ConnectSubTaskError::Unknown)?;

    Ok(())
}
