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
    async fn 全てのサブタスクを取得できる(db: Db) -> AppResult<()> {
        let test = AppTest::new(&db).await?;
        let user = test.login(None).await?;

        let task_a1 = task_factory::create_with_user(&db, &user.id).await?;
        let task_b1 = task_factory::create_subtask(&db, &user.id, &task_a1.id).await?;
        let task_b2 = task_factory::create_subtask(&db, &user.id, &task_a1.id).await?;
        let task_b3 = task_factory::create_subtask(&db, &user.id, &task_a1.id).await?;
        let task_c1 = task_factory::create_subtask(&db, &user.id, &task_b1.id).await?;
        let task_c2 = task_factory::create_subtask(&db, &user.id, &task_b1.id).await?;

        let tasks: Vec<Task> = test.server().get(&TaskPaths::tasks()).await.json();
        assert_eq!(tasks.len(), 6);

        let a1 = tasks.iter().find(|t| t.id == task_a1.id).unwrap();
        assert_eq!(a1.subtask_ids.len(), 3);
        assert!([&task_b1.id, &task_b2.id, &task_b3.id]
            .iter()
            .all(|i| a1.subtask_ids.contains(i)));

        let b1 = tasks.iter().find(|t| t.id == task_b1.id).unwrap();
        assert_eq!(b1.subtask_ids.len(), 2);
        assert!([&task_c1.id, &task_c2.id]
            .iter()
            .all(|i| b1.subtask_ids.contains(i)));

        let b2 = tasks.iter().find(|t| t.id == task_b2.id).unwrap();
        assert_eq!(b2.subtask_ids.len(), 0);

        let b3 = tasks.iter().find(|t| t.id == task_b3.id).unwrap();
        assert_eq!(b3.subtask_ids.len(), 0);

        Ok(())
    }
}
