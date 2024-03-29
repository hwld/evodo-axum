use axum::Json;
use axum_login::tower_sessions::Session;
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::app::AppResult;

use super::login_callback::SIGNUP_USER_ID_KEY;

#[derive(Serialize, ToSchema, Debug)]
pub struct SignupSessionResponse {
    session_exists: bool,
}

#[tracing::instrument(err)]
#[utoipa::path(
    get,
    tag = super::TAG,
    path = super::AuthPaths::signup_session(),
    responses((status = 200, body = SignupSessionResponse))
)]
pub async fn handler(session: Session) -> AppResult<(StatusCode, Json<SignupSessionResponse>)> {
    let session_exists = matches!(session.get::<String>(SIGNUP_USER_ID_KEY).await, Ok(Some(_)));

    Ok((
        StatusCode::OK,
        Json(SignupSessionResponse { session_exists }),
    ))
}
