use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    NotFound,
    Conflict(String),
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Sign in to continue".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong on our side".into(),
            ),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::Conflict("That username or email is already taken".into())
            }
            other => {
                tracing::error!("database error: {other}");
                AppError::Internal
            }
        }
    }
}
