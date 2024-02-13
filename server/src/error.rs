use axum::response::IntoResponse;
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Empty;

// thiserror::ErrorをderiveするとFromの実装でコンフリクトが起こるのでderiveをしていない
// 下から上がってきたErrorを?で返せるようにしたいからFromを実装しているのだが、素直にmap_errorとかを使うべきなんだろうか。
#[derive(Debug)]
pub struct AppError {
    code: StatusCode,
    inner: anyhow::Error,
    body: Option<serde_json::Value>,
}

impl AppError {
    // From<(StatusCode, &str)>を実装したかったが、From<E: Into<anyhow::Error>とコンフリクトするので
    // メソッドで実装する
    pub fn new(code: StatusCode, msg: Option<&str>) -> Self {
        let msg = msg.unwrap_or(code.canonical_reason().unwrap_or("Unknown"));
        AppError {
            code,
            inner: anyhow::anyhow!("{}", msg),
            body: None,
        }
    }

    pub fn with_json<T>(code: StatusCode, json: T) -> Self
    where
        T: Serialize,
    {
        let msg = serde_json::to_string(&json).unwrap_or("Unknown".into());
        AppError {
            code,
            inner: anyhow::anyhow!("{}", msg),
            body: Some(serde_json::to_value(&json).unwrap_or(serde_json::Value::Null)),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, None)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        if let Some(json) = self.body {
            (self.code, json.to_string()).into_response()
        } else {
            // bodyが存在しない場合は内部のエラーメッセージは外に出さない
            (self.code, self.code.canonical_reason().unwrap_or("Unknown")).into_response()
        }
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
            body: None,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
