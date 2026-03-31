use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError{
    #[error("Cache dir not found")]
    NoCacheDirFound,
    #[error("Io error: {0}")]
    CacheIoError(#[from] std::io::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error)
}
