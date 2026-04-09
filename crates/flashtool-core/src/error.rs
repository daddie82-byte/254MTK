//! Error types for the flashtool suite.

use thiserror::Error;

/// Unified error type used across all flashtool crates.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Python environment error: {0}")]
    PythonEnv(String),

    #[error("MTK command failed (exit {code}): {stderr}")]
    MtkCommand { code: i32, stderr: String },

    #[error("Loader not found for chip: {0}")]
    LoaderNotFound(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Zip extraction error: {0}")]
    Zip(String),
}

/// Convenient `Result` alias that uses [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
