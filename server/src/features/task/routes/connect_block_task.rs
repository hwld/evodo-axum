use axum::{extract::State, Json};
use axum_login::AuthSession;

use crate::{
    app::{AppResult, AppState},
    error::AppError,
    features::{
        auth::Auth,
        task::{
            usecases::connect_block_task::{self, ConnectBlockTaskArgs},
            ConnectBlockTask,
        },
    },
};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::TaskPaths::connect_block_task(), responses((status = 200)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<ConnectBlockTask>,
) -> AppResult<()> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    connect_block_task::action(
        &mut tx,
        ConnectBlockTaskArgs {
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
            task::{routes::TaskPaths, test::task_factory, ConnectBlockTask},
            user::test::user_factory,
        },
    };

    #[sqlx::test]
    async fn タスクをブロックタスクにできる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;
        let blocked = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: blocked.id.clone(),
            })
            .await;
        res.assert_status_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_one(&db)
            .await?;
        assert_eq!(block.blocking_task_id, blocking.id);
        assert_eq!(block.blocked_task_id, blocked.id);

        Ok(())
    }

    #[sqlx::test]
    async fn 他人タスクをブロックタスクにできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let other_user = user_factory::create_default(&db).await?;
        let other_user_task = task_factory::create_with_user(&db, &other_user.id).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: other_user_task.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(block.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn 自分自身をブロックできない(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let blocking = task_factory::create_with_user(&db, &user.id).await?;

        let res = test
            .server()
            .post(&TaskPaths::connect_block_task())
            .json(&ConnectBlockTask {
                blocking_task_id: blocking.id.clone(),
                blocked_task_id: blocking.id.clone(),
            })
            .await;
        res.assert_status_not_ok();

        let block = sqlx::query!("SELECT * FROM blocking_tasks;")
            .fetch_all(&db)
            .await?;
        assert!(block.is_empty());

        Ok(())
    }
}
