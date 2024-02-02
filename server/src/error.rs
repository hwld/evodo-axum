use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug)]
pub struct AppError {
    code: StatusCode,
    inner: anyhow::Error,
}

impl AppError {
    // From<(StatusCode, &str)>を実装したかったが、From<E: Into<anyhow::Error>とコンフリクトするので
    // メソッドで実装する
    pub fn new(code: StatusCode, msg: Option<&str>) -> Self {
        let msg = msg.unwrap_or(code.canonical_reason().unwrap_or("Unknown"));
        AppError {
            code,
            inner: anyhow::anyhow!("{}", msg),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, None)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // 内部のエラーメッセージは外に出さない
        (self.code, self.code.canonical_reason().unwrap_or("Unknown")).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: err,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
