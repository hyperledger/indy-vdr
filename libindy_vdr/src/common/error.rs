use std::fmt;

use thiserror::Error;

// use failure::{Backtrace, Context, Fail};

pub mod prelude {
    pub use super::{
        err_msg, input_err, LedgerError, LedgerErrorKind, LedgerResult, LedgerResultExt,
    };
}

#[derive(Debug, Error)]
pub struct LedgerError {
    kind: LedgerErrorKind,
    msg: Option<String>,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    // backtrace (when supported)
}

#[derive(Debug, Error)]
pub enum LedgerErrorKind {
    // General errors
    #[error("Configuration error")]
    Config,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Input error")]
    Input,
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Network error")]
    Network,
    #[error("Resource error")]
    Resource,
    // Transaction errors
    #[error("No consensus from verifiers")]
    PoolNoConsensus,
    #[error("Pool timeout")]
    PoolTimeout,
    #[error("Request failed")]
    PoolRequestFailed(String),
    #[error("Unavailable")]
    Unavailable,
}

impl LedgerError {
    pub fn new(
        kind: LedgerErrorKind,
        msg: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self { kind, msg, source }
    }

    pub fn kind(self) -> LedgerErrorKind {
        self.kind
    }

    pub fn extra(self) -> Option<String> {
        match self.kind {
            LedgerErrorKind::PoolRequestFailed(ref response) => Some(response.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.kind, &self.msg) {
            (LedgerErrorKind::Input, None) => write!(f, "{}", self.kind),
            (LedgerErrorKind::Input, Some(msg)) => f.write_str(msg),
            (kind, None) => write!(f, "{}", kind),
            (kind, Some(msg)) => write!(f, "{}: {}", kind, msg),
        }?;
        if let Some(ref source) = self.source {
            write!(f, "\n{}", source)?;
        }
        Ok(())
    }
}

impl From<LedgerErrorKind> for LedgerError {
    fn from(kind: LedgerErrorKind) -> LedgerError {
        LedgerError::new(kind, None, None)
    }
}

impl From<zmq::Error> for LedgerError {
    fn from(err: zmq::Error) -> LedgerError {
        LedgerError::new(LedgerErrorKind::Network, None, Some(Box::new(err)))
    }
}

impl<M> From<(LedgerErrorKind, M)> for LedgerError
where
    M: fmt::Display + Send + Sync + 'static,
{
    fn from((kind, msg): (LedgerErrorKind, M)) -> LedgerError {
        LedgerError::new(kind, Some(msg.to_string()), None)
    }
}

pub fn err_msg<M>(kind: LedgerErrorKind, msg: M) -> LedgerError
where
    M: fmt::Display + Send + Sync + 'static,
{
    (kind, msg.to_string()).into()
}

pub fn input_err<M>(msg: M) -> LedgerError
where
    M: fmt::Display + Send + Sync + 'static,
{
    (LedgerErrorKind::Input, msg.to_string()).into()
}

pub type LedgerResult<T> = Result<T, LedgerError>;

pub trait LedgerResultExt<T, E> {
    fn map_err_string(self) -> Result<T, String>;
    fn map_input_err<F, M>(self, mapfn: F) -> LedgerResult<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static;
    fn with_err_msg<M>(self, kind: LedgerErrorKind, msg: M) -> LedgerResult<T>
    where
        M: fmt::Display + Send + Sync + 'static;
    fn with_input_err<M>(self, msg: M) -> LedgerResult<T>
    where
        M: fmt::Display + Send + Sync + 'static;
}

impl<T, E> LedgerResultExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_err_string(self) -> Result<T, String> {
        self.map_err(|err| err.to_string())
    }

    fn map_input_err<F, M>(self, mapfn: F) -> LedgerResult<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            LedgerError::new(
                LedgerErrorKind::Input,
                Some(mapfn().to_string()),
                Some(Box::new(err)),
            )
        })
    }

    fn with_err_msg<M>(self, kind: LedgerErrorKind, msg: M) -> LedgerResult<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| LedgerError::new(kind, Some(msg.to_string()), Some(Box::new(err))))
    }

    fn with_input_err<M>(self, msg: M) -> LedgerResult<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            LedgerError::new(
                LedgerErrorKind::Input,
                Some(msg.to_string()),
                Some(Box::new(err)),
            )
        })
    }
}
