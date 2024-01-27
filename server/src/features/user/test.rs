#[cfg(test)]
pub mod factory {
    use uuid::Uuid;

    use crate::{features::user::User, AppResult, Db};

    impl Default for User {
        fn default() -> Self {
            User {
                id: Uuid::new_v4().into(),
                name: "user".into(),
                profile: "profile".into(),
            }
        }
    }

    pub async fn create(db: &Db, input: Option<User>) -> AppResult<User> {
        let user = input.unwrap_or_default();

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
}
