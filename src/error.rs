use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Debug, Serialize)]
pub enum AppError {
    ServerError(String),
    ValidationError(ValidationErrors),
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
    error: AppError,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ServerError(msg) => write!(f, "internal server error {msg}"),
            AppError::ValidationError(_) => write!(f, "invalid payload"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = match self {
            AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
        };

        let message = Json(ErrorBody {
            message: self.to_string(),
            error: self,
        });

        (code, message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        let msg = err.to_string();
        AppError::ServerError(msg)
    }
}
