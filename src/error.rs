use std::io;

use pyo3::PyErr;
use pyo3::exceptions::{PyIOError, PyNotImplementedError, PyRuntimeError, PyValueError};
use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum KTalkError {
    #[error("authentication token is empty")]
    EmptyAuthToken,
    #[error("invalid cookie bundle: {0}")]
    InvalidCookieBundle(String),
    #[error(
        "missing session token for realtime auth; provide it explicitly or capture it from browser traffic"
    )]
    MissingSessionToken,
    #[error("invalid room link: {0}")]
    InvalidRoomLink(String),
    #[error("unsupported KTalk host: {0}")]
    InvalidKTalkHost(String),
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json parsing failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("websocket operation failed: {0}")]
    WebSocket(#[from] tungstenite::Error),
    #[error("http request construction failed: {0}")]
    HttpRequest(#[from] http::Error),
    #[error("io operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("audio publishing is not implemented yet: full WebRTC media publishing is required")]
    UnsupportedAudioPublishing,
}

pub type Result<T> = std::result::Result<T, KTalkError>;

impl From<KTalkError> for PyErr {
    fn from(value: KTalkError) -> Self {
        match value {
            KTalkError::Io(_) => PyIOError::new_err(value.to_string()),
            KTalkError::EmptyAuthToken
            | KTalkError::InvalidCookieBundle(_)
            | KTalkError::MissingSessionToken
            | KTalkError::InvalidRoomLink(_)
            | KTalkError::InvalidKTalkHost(_) => PyValueError::new_err(value.to_string()),
            KTalkError::UnsupportedAudioPublishing => {
                PyNotImplementedError::new_err(value.to_string())
            }
            KTalkError::Http(_)
            | KTalkError::Json(_)
            | KTalkError::WebSocket(_)
            | KTalkError::HttpRequest(_)
            | KTalkError::Protocol(_) => PyRuntimeError::new_err(value.to_string()),
        }
    }
}
