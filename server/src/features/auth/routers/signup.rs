use super::login_callback::SIGNUP_USER_ID_KEY;
use crate::{
    features::{auth::Auth, user::User},
    AppResult, AppState,
};
use axum::{extract::State, response::IntoResponse, Json};
use axum_login::{tower_sessions::Session, AuthSession};
use garde::{Unvalidated, Validate};
use http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Debug, Validate)]
pub struct CreateUser {
    #[garde(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,

    #[garde(skip)]
    #[schema(max_length = 500)]
    pub profile: String,
}

#[tracing::instrument(err)]
#[utoipa::path(post, tag = "auth", path = "/signup", request_body = CreateUser, responses((status = 201, body = User)))]
pub async fn handler(
    mut auth_session: AuthSession<Auth>,
    session: Session,
    State(AppState { db }): State<AppState>,
    Json(payload): Json<Unvalidated<CreateUser>>,
) -> AppResult<impl IntoResponse> {
    let input = payload.validate(&())?;
    let Ok(Some(user_id)) = session.get::<String>(SIGNUP_USER_ID_KEY).await else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users(id, name, profile) VALUES($1, $2, $3) RETURNING *",
        user_id,
        input.name,
        input.profile,
    )
    .fetch_one(&db)
    .await?;

    session.flush().await?;
    auth_session.login(&user).await?;

    Ok((StatusCode::CREATED, Json(user)).into_response())
}
