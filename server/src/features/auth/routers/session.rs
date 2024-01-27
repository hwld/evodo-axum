use crate::{
    features::auth::{Auth, Session},
    AppResult,
};
use axum::Json;
use axum_login::AuthSession;
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct SessionResponse {
    session: Option<Session>,
}

#[tracing::instrument(err)]
#[utoipa::path(get, tag = super::TAG, path = super::Paths::session(), responses((status = 200, body = SessionResponse)))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
) -> AppResult<(StatusCode, Json<SessionResponse>)> {
    let session = auth_session.user.map(|user| Session { user });
    Ok((StatusCode::OK, Json(SessionResponse { session })))
}
