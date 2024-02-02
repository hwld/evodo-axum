#[cfg(test)]
pub mod user_factory {
    use uuid::Uuid;

    use crate::{app::AppResult, app::Db, features::user::User};

    impl Default for User {
        fn default() -> Self {
            User {
                id: Uuid::new_v4().into(),
                name: "user".into(),
                profile: "profile".into(),
            }
        }
    }

    async fn create_inner(db: &Db, user: User) -> AppResult<User> {
        let created = sqlx::query_as!(
            User,
            "INSERT INTO users(id, name, profile) VALUES($1, $2, $3) RETURNING * ;",
            user.id,
            user.name,
            user.profile
        )
        .fetch_one(db)
        .await?;

        Ok(created)
    }

    pub async fn create_default(db: &Db) -> AppResult<User> {
        create_inner(db, User::default()).await
    }
}
