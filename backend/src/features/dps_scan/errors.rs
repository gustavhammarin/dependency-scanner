use axum::response::IntoResponse;
use reqwest::StatusCode;
use thiserror::Error;

use crate::{features::package_service::errors::PackageFetcherError, features::zip_source::ZipSourceError};

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Zip source error: {0}")]
    ZipSource(#[from] ZipSourceError),

    #[error("Package fetch error: {0}")]
    PackageFetchError(#[from] PackageFetcherError),
}

impl IntoResponse for ScanError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error = ?self, "ScanError");
        let status = match &self {
            ScanError::ZipSource(_) | ScanError::PackageFetchError(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
