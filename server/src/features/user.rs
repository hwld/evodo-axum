use axum_login::AuthUser;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub profile: String,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.id.as_bytes()
    }
}
