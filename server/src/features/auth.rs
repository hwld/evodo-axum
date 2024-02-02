pub mod routes;
pub mod test;

use self::routes::AuthPaths;

use super::user::User;
use crate::{app::Db, config::Env};
use anyhow::anyhow;
use axum::async_trait;
use axum_login::{AuthnBackend, UserId};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope,
};
pub use routes::router;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct Session {
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
    pub nonce: Nonce,
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub db: Db,
    pub client: CoreClient,
}

impl Auth {
    pub async fn new(db: Db) -> Self {
        let google_client_id = ClientId::new(Env::google_client_id());
        let google_client_secret = ClientSecret::new(Env::google_client_secret());

        let issuer_url =
            IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .expect("Failed");

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            google_client_id,
            Some(google_client_secret),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!(
                "{}{}",
                Env::base_url(),
                AuthPaths::login_callback()
            ))
            .expect("Invalid redirect URL"),
        );

        Auth { db, client }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken, Nonce) {
        self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".into()))
            .url()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),

    // 新規登録の場合に返すエラー
    // authenticateがResult<User, Error>を返すのでOkとして返せない。
    // けど、authenticateがErrを返すとログに出力されちゃう・・・
    #[error("User not found")]
    UserNotFound(UserId<Auth>),
}

#[async_trait]
impl AuthnBackend for Auth {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if credentials.old_state.secret() != credentials.new_state.secret() {
            return Ok(None);
        }

        let token_res = self
            .client
            .exchange_code(AuthorizationCode::new(credentials.code))
            .request_async(openidconnect::reqwest::async_http_client)
            .await
            .map_err(|e| AuthError::Unknown(e.into()))?;

        let token_verifier = self.client.id_token_verifier();
        let id_token_claims = token_res
            .extra_fields()
            .id_token()
            .ok_or_else(|| anyhow!("Server did not return an ID token"))?
            .claims(&token_verifier, &credentials.nonce)
            .map_err(|e| AuthError::Unknown(e.into()))?;

        let user_id = id_token_claims.subject().to_string();

        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(&self.db)
            .await
            .map_err(|_| AuthError::UserNotFound(user_id))?;

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(Self::User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AuthError::Unknown(e.into()))?;

        Ok(user)
    }
}
