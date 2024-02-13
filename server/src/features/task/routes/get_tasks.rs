use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use http::StatusCode;

use crate::app::AppResult;
use crate::features::task::db::find_tasks;
use crate::{app::AppState, error::AppError, features::auth::Auth};

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::TaskPaths::tasks(), responses((status = 200, body = [Task])))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    State(AppState { db }): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = auth_session.user else {
        return Err(AppError::unauthorized());
    };

    let mut tx = db.begin().await?;

    let tasks = find_tasks(&mut tx, &user.id).await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(tasks)).into_response())
}

#[cfg(test)]
mod tests {

    use crate::app::tests::AppTest;
    use crate::app::Db;
    use crate::features::task::routes::TaskPaths;
    use crate::features::task::Task;
    use crate::features::{task::test::task_factory, user::test::user_factory};

    use super::*;

    #[sqlx::test]
    async fn 自分の全てのタスクを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;

        let other_user = user_factory::create_default(&db).await?;
        task_factory::create_with_user(&db, &other_user.id).await?;

        let user = test.login(None).await?;
        tokio::try_join!(
            task_factory::create_with_user(&db, &user.id),
            task_factory::create_with_user(&db, &user.id),
            task_factory::create_with_user(&db, &user.id),
        )?;

        let tasks: Vec<Task> = test.server().get(&TaskPaths::tasks()).await.json();
        assert_eq!(tasks.len(), 3);

        Ok(())
    }

    #[sqlx::test]
    async fn すべてのサブタスクとブロックされたタスクを重複なく取得できる(
        db: Db,
    ) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task1 = task_factory::create_with_user(&db, &user.id).await?;
        let task11 = task_factory::create_default_blocked_task(&db, &user.id, &task1.id).await?;
        let task12 = task_factory::create_default_blocked_task(&db, &user.id, &task1.id).await?;
        let task13 = task_factory::create_default_blocked_task(&db, &user.id, &task1.id).await?;
        let task14 = task_factory::create_default_sub_task(&db, &user.id, &task1.id).await?;
        let task15 = task_factory::create_default_sub_task(&db, &user.id, &task1.id).await?;

        let tasks: Vec<Task> = test.server().get(&TaskPaths::tasks()).await.json();
        assert_eq!(tasks.len(), 6);

        let t1 = tasks.iter().find(|t| t.id == task1.id).unwrap();
        assert_eq!(t1.blocked_task_ids.len(), 3);
        assert!([&task11.id, &task12.id, &task13.id]
            .iter()
            .all(|d| t1.blocked_task_ids.contains(d)));

        assert_eq!(t1.sub_task_ids.len(), 2);
        assert!([&task14.id, &task15.id]
            .iter()
            .all(|i| t1.sub_task_ids.contains(i)));

        Ok(())
    }
}
