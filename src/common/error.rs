use std::{error::Error as StdError, fmt::Write as _};

use axum::{
    BoxError,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use deadpool_postgres::PoolError;
use thiserror::Error;
use tracing::error;

use crate::common::dto::RestApiResponse;

use super::dto::ApiResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] PoolError), // Used for database-related errors

    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] tokio_postgres::Error),

    #[error("Not found: {0}")]
    NotFound(String), // Used for not found errors

    #[error("Internal server error")]
    InternalError,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Forbidden Request")]
    Forbidden,

    /// Used for file-related errors
    #[error("File data is empty")]
    InvalidFileData,

    #[error("File size exceeded")]
    FileSizeExceeded,

    #[error("Invalid file name")]
    InvalidFileName,

    #[error("Unsupported file extension")]
    UnsupportedFileExtension,

    /// Used for authentication-related errors
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Missing credentials")]
    MissingCredentials,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User not found")]
    UserNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) | AppError::DatabaseQueryError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::InvalidFileData
            | AppError::FileSizeExceeded
            | AppError::InvalidFileName
            | AppError::UnsupportedFileExtension => StatusCode::BAD_REQUEST,
            AppError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AppError::MissingCredentials => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
        };

        log_app_error(&self, status);

        let body = axum::Json(ApiResponse::<()> {
            status: status.as_u16(),
            message: self.to_string(),
            data: None,
        });

        (status, body).into_response()
    }
}

fn log_app_error(error: &AppError, status: StatusCode) {
    if !status.is_server_error() {
        return;
    }

    error!(
        status = status.as_u16(),
        error = %error,
        error_debug = ?error,
        source_chain = %format_error_sources(error),
        "Application error"
    );
}

fn format_error_sources(error: &(dyn StdError + 'static)) -> String {
    let mut sources = String::new();
    let mut current = error.source();
    let mut index = 0;

    while let Some(source) = current {
        if index > 0 {
            sources.push_str(" | ");
        }

        let _ = write!(sources, "#{index}: {source}");
        current = source.source();
        index += 1;
    }

    if sources.is_empty() {
        "none".into()
    } else {
        sources
    }
}

/// handle_error is a function that middlewares the error handling in the application.
/// It takes a BoxError as input and returns an HTTP response.
/// It maps the error to an appropriate HTTP status code and constructs a JSON response body.
/// The function is used to handle errors that occur during the request processing.
/// It is designed to be used with the axum framework.
pub async fn handle_error(error: BoxError) -> impl IntoResponse {
    let status = if error.is::<tower::timeout::error::Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    let message = error.to_string();
    error!(?status, %message, "Request failed");

    let body = RestApiResponse::<()>::failure(status.as_u16(), message);

    (status, body)
}
