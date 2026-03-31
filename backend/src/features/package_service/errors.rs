use axum::response::IntoResponse;
use reqwest::StatusCode;
use thiserror::Error;

use crate::features::{disk_cache::error::CacheError, zip_source::ZipSourceError};

#[derive(Debug, Error)]
pub enum PackageFetcherError{
    #[error("Fetch Error: {0}")]
    FetchError(#[from] ZipSourceError),
    #[error("Db Error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError)
}

impl IntoResponse for PackageFetcherError{
    fn into_response(self) -> axum::response::Response {
        let status = match &self{
            PackageFetcherError::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PackageFetcherError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PackageFetcherError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
