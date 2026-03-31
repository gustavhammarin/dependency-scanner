use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZipSourceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    #[error("Decompiler not found — {0}")]
    DecompilerNotFound(String),
    #[error("Archive validation error: {0}")]
    Validation(String),
    #[error("Repository root not found in extracted archive")]
    RepoRootNotFound,
    #[error("tokio task error: {0}")]
    TokioTaskError(String),
    #[error("Source url not found")]
    SourceUrlNotFound,
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
