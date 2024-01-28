use crate::AppResult;
use dotenv::dotenv;
use std::env;

pub fn load() {
    dotenv().ok();
}

pub fn database_url() -> AppResult<String> {
    Ok(env::var("DATABASE_URL")?)
}

pub fn google_client_id() -> AppResult<String> {
    Ok(env::var("GOOGLE_CLIENT_ID")?)
}

pub fn google_client_secret() -> AppResult<String> {
    Ok(env::var("GOOGLE_CLIENT_SECRET")?)
}
