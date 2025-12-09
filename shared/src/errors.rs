use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

/// Application error types for consistent error handling across services
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Internal server error (500)
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Bad request error (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Unauthorized error (401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden error (403)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Not found error (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict error (409)
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            AppError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub type AppResult<T> = Result<T, AppError>;
