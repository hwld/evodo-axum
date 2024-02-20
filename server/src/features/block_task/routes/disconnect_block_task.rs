use axum::{extract::State, Json};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        block_task::{
            usecases::disconnect_block_task::{self, DisconnectBlockTaskArgs},
            DisconnectBlockTask,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(
    delete,
    tag = super::TAG,
    path = super::BlockTaskPaths::disconnect_block_task(),
    responses(( status = 200))
)]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<DisconnectBlockTask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    disconnect_block_task::action(
        &mut tx,
        DisconnectBlockTaskArgs {
            blocking_task_id: &payload.blocking_task_id,
            blocked_task_id: &payload.blocked_task_id,
            user_id: &user.id,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{tests::AppTest, AppResult, Db},
        features::{
            block_task::{routes::BlockTaskPaths, DisconnectBlockTask},
            task::test::task_factory,
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn ブロックタスク関係を削除できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked =
            task_factory::create_default_blocked_task(&db, &user.id, &blocking.id).await?;

        let res = test
            .server()
            .delete(&BlockTaskPaths::disconnect_block_task())
            .json(&DisconnectBlockTask {
                blocking_task_id: blocking.id,
                blocked_task_id: blocked.id,
            })
            .await;
        res.assert_status_ok();

        let sub_tasks = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(sub_tasks.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 他のユーザーのブロックタスク関係は削除できない(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_blocking = task_factory::create_with_user(&db, &other_user.id).await?;
        let other_user_blocked =
            task_factory::create_default_blocked_task(&db, &other_user.id, &other_user_blocking.id)
                .await?;

        test.login(None).await?;
        let res = test
            .server()
            .delete(&BlockTaskPaths::disconnect_block_task())
            .json(&DisconnectBlockTask {
                blocking_task_id: other_user_blocking.id,
                blocked_task_id: other_user_blocked.id,
            })
            .await;
        res.assert_status_not_ok();

        let sub_tasks = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(!sub_tasks.is_empty());

        Ok(())
    }
}
