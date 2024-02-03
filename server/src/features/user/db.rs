use crate::app::{AppResult, Connection};

use super::User;

pub async fn find_user(db: &mut Connection, user_id: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1;", user_id)
        .fetch_optional(&mut *db)
        .await?;

    Ok(user)
}

pub struct InsertUserArgs<'a> {
    pub user_id: &'a str,
    pub name: &'a str,
    pub profile: &'a str,
}
pub async fn insert_user<'a>(db: &mut Connection, args: InsertUserArgs<'a>) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users(id, name, profile) VALUES($1, $2, $3) RETURNING *",
        args.user_id,
        args.name,
        args.profile
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(user)
}
