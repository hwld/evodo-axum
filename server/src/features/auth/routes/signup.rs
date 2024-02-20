use super::login_callback::SIGNUP_USER_ID_KEY;
use crate::app::AppResult;
use crate::features::user::db::{insert_user, InsertUserArgs};
use crate::{app::AppState, error::AppError, features::auth::Auth};
use axum::{extract::State, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_login::{tower_sessions::Session, AuthSession};
use garde::Validate;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate)]
pub struct CreateUser {
    #[garde(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,

    #[garde(skip)]
    #[schema(max_length = 500)]
    pub profile: String,
}

#[tracing::instrument(err, skip(auth_session, session, db))]
#[utoipa::path(
    post,
    tag = super::TAG,
    path = super::AuthPaths::signup(),
    request_body = CreateUser,
    responses((status = 201, body = User))
)]
pub async fn handler(
    mut auth_session: AuthSession<Auth>,
    session: Session,
    State(AppState { db }): State<AppState>,
    WithValidation(payload): WithValidation<Json<CreateUser>>,
) -> AppResult<impl IntoResponse> {
    let Ok(Some(user_id)) = session.get::<String>(SIGNUP_USER_ID_KEY).await else {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            Some("Signup session not found"),
        ));
    };

    let mut conn = db.begin().await?;
    let user = insert_user(
        &mut conn,
        InsertUserArgs {
            user_id: &user_id,
            name: &payload.name,
            profile: &payload.profile,
        },
    )
    .await?;

    conn.commit().await?;

    session.flush().await?;
    auth_session.login(&user).await?;

    Ok((StatusCode::CREATED, Json(user)).into_response())
}
