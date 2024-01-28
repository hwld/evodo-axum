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
        Self::signup_page();
        Self::auth_error_page();
    }

    pub fn port() -> String {
        Self::get_env("PORT")
    }

    pub fn base_url() -> String {
        Self::get_env("BASE_URL")
    }

    pub fn client_url() -> String {
        Self::get_env("CLIENT_URL")
    }

    pub fn database_url() -> String {
        Self::get_env("DATABASE_URL")
    }

    pub fn google_client_id() -> String {
        Self::get_env("GOOGLE_CLIENT_ID")
    }

    pub fn google_client_secret() -> String {
        Self::get_env("GOOGLE_CLIENT_SECRET")
    }

    pub fn signup_page() -> String {
        Self::get_env("SIGNUP_PAGE")
    }

    pub fn auth_error_page() -> String {
        Self::get_env("AUTH_ERROR_PAGE")
    }

    fn get_env(key: &str) -> String {
        let error_message = format!("Failed to load {}", key);
        env::var(key).expect(&error_message)
    }
}
