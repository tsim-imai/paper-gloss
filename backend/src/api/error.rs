use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Application errors
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    UnprocessableEntity(String),
    GatewayTimeout(String),
    Conflict(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
            AppError::GatewayTimeout(msg) => write!(f, "Gateway timeout: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_name, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NotFound", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BadRequest", msg),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", msg)
            }
            AppError::UnprocessableEntity(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "UnprocessableEntity", msg)
            }
            AppError::GatewayTimeout(msg) => {
                (StatusCode::GATEWAY_TIMEOUT, "GatewayTimeout", msg)
            }
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg),
        };

        let body = ErrorResponse {
            error: error_name.to_string(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

/// Convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Internal error: {:?}", err);
        AppError::InternalServerError(err.to_string())
    }
}

/// Convert sqlx::Error to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::InternalServerError("Database error".to_string()),
        }
    }
}
