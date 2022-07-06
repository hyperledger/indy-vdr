use std::fmt;

use serde_json;

use thiserror::Error;

/// Common set of error module exports
pub mod prelude {
    pub use super::{err_msg, input_err, VdrError, VdrErrorKind, VdrResult, VdrResultExt};
}

/// Library error type
#[derive(Debug, Error)]
pub struct VdrError {
    kind: VdrErrorKind,
    msg: Option<String>,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    // backtrace (when supported)
}

/// Supported error kinds for `VdrError`
#[derive(Debug, Error)]
pub enum VdrErrorKind {
    // General errors
    #[error("Configuration error")]
    Config,
    #[error("Connection error")]
    Connection,
    #[error("File system error: {0}")]
    FileSystem(std::io::Error),
    #[error("Input error")]
    Input,
    #[error("Resource error")]
    Resource,
    #[error("Service unavailable")]
    Unavailable,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Incompatible error")]
    Incompatible,
    // Transaction errors
    #[error("No consensus from verifiers")]
    PoolNoConsensus,
    #[error("Request failed: {}", pool_request_failed_reason(.0))]
    PoolRequestFailed(String),
    #[error("Pool timeout")]
    PoolTimeout,
}

impl VdrError {
    pub fn new(
        kind: VdrErrorKind,
        msg: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self { kind, msg, source }
    }

    pub fn kind(&self) -> &VdrErrorKind {
        &self.kind
    }

    pub fn extra(&self) -> Option<String> {
        match self.kind {
            VdrErrorKind::PoolRequestFailed(ref response) => Some(response.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for VdrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.kind, &self.msg) {
            (VdrErrorKind::Input, None) => write!(f, "{}", self.kind),
            (VdrErrorKind::Input, Some(msg)) => f.write_str(msg),
            (kind, None) => write!(f, "{}", kind),
            (kind, Some(msg)) => write!(f, "{}: {}", kind, msg),
        }?;
        if let Some(ref source) = self.source {
            write!(f, "\n{}", source)?;
        }
        Ok(())
    }
}

impl From<VdrError> for VdrErrorKind {
    fn from(error: VdrError) -> VdrErrorKind {
        error.kind
    }
}

impl From<VdrErrorKind> for VdrError {
    fn from(kind: VdrErrorKind) -> VdrError {
        VdrError::new(kind, None, None)
    }
}

impl From<crate::utils::ConversionError> for VdrError {
    fn from(err: crate::utils::ConversionError) -> Self {
        VdrError::new(VdrErrorKind::Input, Some(err.to_string()), None)
    }
}

impl From<crate::utils::ValidationError> for VdrError {
    fn from(err: crate::utils::ValidationError) -> Self {
        VdrError::new(VdrErrorKind::Input, Some(err.to_string()), None)
    }
}

impl From<zmq::Error> for VdrError {
    fn from(err: zmq::Error) -> VdrError {
        VdrError::new(VdrErrorKind::Connection, None, Some(Box::new(err)))
    }
}

impl<M> From<(VdrErrorKind, M)> for VdrError
where
    M: fmt::Display + Send + Sync + 'static,
{
    fn from((kind, msg): (VdrErrorKind, M)) -> VdrError {
        VdrError::new(kind, Some(msg.to_string()), None)
    }
}

/// Construct a `VdrError` from an error kind and message
pub fn err_msg<M>(kind: VdrErrorKind, msg: M) -> VdrError
where
    M: fmt::Display + Send + Sync + 'static,
{
    (kind, msg.to_string()).into()
}

/// Construct a `VdrError` input error from an error message
pub fn input_err<M>(msg: M) -> VdrError
where
    M: fmt::Display + Send + Sync + 'static,
{
    (VdrErrorKind::Input, msg.to_string()).into()
}

/// Library result type
pub type VdrResult<T> = Result<T, VdrError>;

/// Trait providing utility methods for handling other result types
pub trait VdrResultExt<T, E> {
    fn map_err_string(self) -> Result<T, String>;
    fn map_input_err<F, M>(self, mapfn: F) -> VdrResult<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static;
    fn with_err_msg<M>(self, kind: VdrErrorKind, msg: M) -> VdrResult<T>
    where
        M: fmt::Display + Send + Sync + 'static;
    fn with_input_err<M>(self, msg: M) -> VdrResult<T>
    where
        M: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VdrResultExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_err_string(self) -> Result<T, String> {
        self.map_err(|err| err.to_string())
    }

    fn map_input_err<F, M>(self, mapfn: F) -> VdrResult<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            VdrError::new(
                VdrErrorKind::Input,
                Some(mapfn().to_string()),
                Some(Box::new(err)),
            )
        })
    }

    fn with_err_msg<M>(self, kind: VdrErrorKind, msg: M) -> VdrResult<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| VdrError::new(kind, Some(msg.to_string()), Some(Box::new(err))))
    }

    fn with_input_err<M>(self, msg: M) -> VdrResult<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            VdrError::new(
                VdrErrorKind::Input,
                Some(msg.to_string()),
                Some(Box::new(err)),
            )
        })
    }
}

fn pool_request_failed_reason(reply: &str) -> String {
    if let Ok(reply) = serde_json::from_str::<serde_json::Value>(reply) {
        if let Some(reason) = reply["reason"].as_str() {
            return reason.to_owned();
        }
    }
    "Unknown reason".to_owned()
}
