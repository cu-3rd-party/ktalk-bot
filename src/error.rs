use std::io;

use pyo3::PyErr;
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KTalkError {
    #[error("authentication file not found: {path}")]
    AuthFileNotFound { path: String },
    #[error("authentication token is empty")]
    EmptyAuthToken,
    #[error("invalid room link: {0}")]
    InvalidRoomLink(String),
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json parsing failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io operation failed: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, KTalkError>;

impl From<KTalkError> for PyErr {
    fn from(value: KTalkError) -> Self {
        match value {
            KTalkError::AuthFileNotFound { .. } | KTalkError::Io(_) => {
                PyIOError::new_err(value.to_string())
            }
            KTalkError::EmptyAuthToken | KTalkError::InvalidRoomLink(_) => {
                PyValueError::new_err(value.to_string())
            }
            KTalkError::Http(_) | KTalkError::Json(_) => PyRuntimeError::new_err(value.to_string()),
        }
    }
}
