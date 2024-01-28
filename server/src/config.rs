use dotenv::dotenv;
use std::env;

pub struct Env;
impl Env {
    pub fn load() {
        dotenv().ok();

        // 全部の環境変数が読み込めるか試す
        Self::port();
        Self::base_url();
        Self::client_url();
        Self::database_url();
        Self::google_client_id();
        Self::google_client_secret();
    }

    pub fn port() -> String {
        env::var("PORT").expect("Failed to load PORT")
    }

    pub fn base_url() -> String {
        env::var("BASE_URL").expect("Failed to load BASE_URL")
    }

    pub fn client_url() -> String {
        env::var("CLIENT_URL").expect("Failed to load CLIENT_URL")
    }

    pub fn database_url() -> String {
        env::var("DATABASE_URL").expect("Failed to load DATABASE_URL")
    }

    pub fn google_client_id() -> String {
        env::var("GOOGLE_CLIENT_ID").expect("Failed to load GOOGLE_CLIENIT_ID")
    }

    pub fn google_client_secret() -> String {
        env::var("GOOGLE_CLIENT_SECRET").expect("Failed to load GOOGLE_CLIEINT_SECRET")
    }
}
