use axum::response::{IntoResponse, Response};
use http::StatusCode;
use maud::html;
use thiserror::Error;
use tracing::log::error;

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct ServerError(pub anyhow::Error);

/// 404 Handler, only activates on non-hx requests.
pub async fn page_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        html! {
            "The following page was not found!"
        },
    )
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        error!("Server error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}



// NEW error handling method, needs implementing still.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Internal Server Error: {0}")]
    Internal(anyhow::Error),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Bad Request: {}", msg),
                )
                    .into_response()
            }
            AppError::Unauthorized(msg) => {
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Unauthorized: {}", msg),
                )
                    .into_response()
            }
            AppError::Forbidden(msg) => {
                (
                    StatusCode::FORBIDDEN,
                    format!("Forbidden: {}", msg),
                )
                    .into_response()
            }
            AppError::NotFound(msg) => {
                (
                    StatusCode::NOT_FOUND,
                    format!("Not Found: {}", msg),
                )
                    .into_response()
            }
            AppError::Internal(err) => {
                error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error: {}", err),
                )
                    .into_response()
            }
            AppError::ServiceUnavailable(msg) => {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Service Unavailable: {}", msg),
                )
                    .into_response()
            }
        }
    }
}

impl AppError {
    // Helper function to create an Internal error from any anyhow::Error
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<anyhow::Error>
    {
        AppError::Internal(err.into())
    }
}