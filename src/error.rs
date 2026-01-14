use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    WazuhConnection(String),
    WazuhQuery(String),
    ParseError(String),
    Serialization(String),
    Unauthorized,
    SessionExpired,
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::WazuhConnection(msg) => write!(f, "Wazuh connection error: {}", msg),
            AppError::WazuhQuery(msg) => write!(f, "Wazuh query error: {}", msg),
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized access"),
            AppError::SessionExpired => write!(f, "Session expired"),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let error_message = self.to_string();

        match self {
            AppError::Unauthorized | AppError::SessionExpired => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "error": error_message
                }))
            }
            AppError::NotFound(_) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "error": error_message
                }))
            }
            AppError::BadRequest(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "error": error_message
                }))
            }
            _ => {
                tracing::error!("Internal error: {}", error_message);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": "Internal server error"
                }))
            }
        }
    }
}