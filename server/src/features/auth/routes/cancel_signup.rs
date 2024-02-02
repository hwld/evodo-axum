use axum::response::IntoResponse;
use axum_login::tower_sessions::Session;
use http::StatusCode;

use crate::app::AppResult;

use super::login_callback::SIGNUP_USER_ID_KEY;

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::AuthPaths::cancel_signup(), responses((status = 200)))]
pub async fn handler(session: Session) -> AppResult<impl IntoResponse> {
    if let Ok(Some(_)) = session.get::<String>(SIGNUP_USER_ID_KEY).await {
        session.flush().await?;
    }

    Ok(StatusCode::OK)
}
