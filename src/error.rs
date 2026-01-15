//! Defines the custom error types for the application.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// The primary error type for this application, designed to be easily convertible into an HTTP response.
#[derive(Debug, Error)]
pub enum AppError {
    /// Represents a failure from the database.
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Represents an error during the code compilation/auditing process.
    #[error("Audit error: {0}")]
    Audit(String),

    /// Represents a failure to find a required resource.
    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Sqlx(e) => {
                // Log the full error for debugging, but return a generic message to the client.
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal database error occurred".to_string(),
                )
            }
            AppError::Audit(e) => (StatusCode::BAD_REQUEST, e),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
