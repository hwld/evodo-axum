use crate::{
    features::{auth::Auth, user::User},
    AppResult,
};
use axum::Json;
use axum_login::AuthSession;
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct SessionResponse {
    user: Option<User>,
}

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::SESSION_PATH, responses((status = 200, body = SessionResponse)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
) -> AppResult<(StatusCode, Json<SessionResponse>)> {
    Ok((
        StatusCode::OK,
        Json(SessionResponse {
            user: auth_session.user,
        }),
    ))
}
