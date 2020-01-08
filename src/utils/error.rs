extern crate failure;

use std::fmt;
use std::io;

use failure::{Backtrace, Context, Fail};

pub mod prelude {
    pub use super::{
        err_msg, Context, LedgerError, LedgerErrorExt, LedgerErrorKind, LedgerResult,
        LedgerResultExt,
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LedgerErrorKind {
    // Common errors
    #[fail(display = "Invalid library state")]
    InvalidState,
    #[fail(display = "Invalid structure")]
    InvalidStructure,
    #[fail(display = "IO error")]
    IOError,
    // Ledger errors
    #[fail(display = "No consensus")]
    NoConsensus,
    #[fail(display = "Invalid transaction")]
    InvalidTransaction,
    #[fail(display = "Item not found on ledger")]
    ItemNotFound,
    // Pool errors
    #[fail(display = "Invalid pool handle")]
    PoolHandleInvalid,
    #[fail(display = "Pool work terminated")]
    PoolTerminated,
    #[fail(display = "Pool timeout")]
    PoolTimeout,
    #[fail(display = "Pool Genesis Transactions are not compatible with Protocol version")]
    PoolIncompatibleProtocolVersion,
}

#[derive(Debug)]
pub struct LedgerError {
    inner: Context<LedgerErrorKind>,
}

impl Fail for LedgerError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl LedgerError {
    pub fn from_msg<D>(kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        LedgerError {
            inner: Context::new(msg).context(kind),
        }
    }

    pub fn kind(&self) -> LedgerErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> LedgerError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        let kind = self.kind();
        LedgerError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }

    pub fn map<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        LedgerError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }
}

pub fn err_msg<D>(kind: LedgerErrorKind, msg: D) -> LedgerError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    LedgerError::from_msg(kind, msg)
}

impl From<LedgerErrorKind> for LedgerError {
    fn from(kind: LedgerErrorKind) -> LedgerError {
        LedgerError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<LedgerErrorKind>> for LedgerError {
    fn from(inner: Context<LedgerErrorKind>) -> LedgerError {
        LedgerError { inner: inner }
    }
}

impl From<io::Error> for LedgerError {
    fn from(err: io::Error) -> Self {
        err.context(LedgerErrorKind::IOError).into()
    }
}

impl From<zmq::Error> for LedgerError {
    fn from(err: zmq::Error) -> Self {
        err.context(LedgerErrorKind::IOError).into()
    }
}

pub type LedgerResult<T> = Result<T, LedgerError>;

/// Extension methods for `Result`.
pub trait LedgerResultExt<T, E> {
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerResult<T>
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> LedgerResultExt<T, E> for Result<T, E>
where
    E: Fail,
{
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerResult<T>
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait LedgerErrorExt {
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<E> LedgerErrorExt for E
where
    E: Fail,
{
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.context(msg).context(kind).into()
    }
}
/*
impl From<LedgerErrorKind> for ErrorCode {
    fn from(code: LedgerErrorKind) -> ErrorCode {
        match code {
            LedgerErrorKind::InvalidState => ErrorCode::CommonInvalidState,
            LedgerErrorKind::InvalidStructure => ErrorCode::CommonInvalidStructure,
            LedgerErrorKind::IOError => ErrorCode::CommonIOError,
            LedgerErrorKind::NoConsensus => ErrorCode::LedgerNoConsensusError,
            LedgerErrorKind::InvalidTransaction => ErrorCode::LedgerInvalidTransaction,
            LedgerErrorKind::LedgerItemNotFound => ErrorCode::LedgerNotFound,
            LedgerErrorKind::PoolNotCreated => ErrorCode::PoolLedgerNotCreatedError,
            LedgerErrorKind::InvalidPoolHandle => ErrorCode::PoolLedgerInvalidPoolHandle,
            LedgerErrorKind::PoolTerminated => ErrorCode::PoolLedgerTerminated,
            LedgerErrorKind::PoolTimeout => ErrorCode::PoolLedgerTimeout,
            LedgerErrorKind::PoolConfigAlreadyExists => ErrorCode::PoolLedgerConfigAlreadyExistsError,
            LedgerErrorKind::PoolIncompatibleProtocolVersion => {
                ErrorCode::PoolIncompatibleProtocolVersion
            }
        }
    }
}*/
/*
impl From<ErrorCode> for LedgerResult<()> {
    fn from(err: ErrorCode) -> LedgerResult<()> {
        if err == ErrorCode::Success {
            Ok(())
        } else {
            Err(err.into())
        }
    }
}

impl From<ErrorCode> for LedgerError {
    fn from(err: ErrorCode) -> LedgerError {
        err_msg(err.into(), "Plugin returned error".to_string())
    }
}

impl From<ErrorCode> for LedgerErrorKind {
    fn from(err: ErrorCode) -> LedgerErrorKind {
        match err {
            ErrorCode::CommonInvalidState => LedgerErrorKind::InvalidState,
            ErrorCode::CommonInvalidStructure => LedgerErrorKind::InvalidStructure,
            ErrorCode::CommonIOError => LedgerErrorKind::IOError,
            ErrorCode::LedgerNoConsensusError => LedgerErrorKind::NoConsensus,
            ErrorCode::LedgerInvalidTransaction => LedgerErrorKind::InvalidTransaction,
            ErrorCode::LedgerNotFound => LedgerErrorKind::LedgerItemNotFound,
            ErrorCode::PoolLedgerNotCreatedError => LedgerErrorKind::PoolNotCreated,
            ErrorCode::PoolLedgerInvalidPoolHandle => LedgerErrorKind::InvalidPoolHandle,
            ErrorCode::PoolLedgerTerminated => LedgerErrorKind::PoolTerminated,
            ErrorCode::PoolLedgerTimeout => LedgerErrorKind::PoolTimeout,
            ErrorCode::PoolLedgerConfigAlreadyExistsError => LedgerErrorKind::PoolConfigAlreadyExists,
            ErrorCode::PoolIncompatibleProtocolVersion => {
                LedgerErrorKind::PoolIncompatibleProtocolVersion
            }
            _code => LedgerErrorKind::InvalidState,
        }
    }
}

pub type LedgerResult<T> = Result<T, LedgerError>;

/// Extension methods for `Result`.
pub trait LedgerResultExt<T, E> {
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerResult<T>
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> LedgerResultExt<T, E> for Result<T, E>
where
    E: Fail,
{
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerResult<T>
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait LedgerErrorExt {
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<E> LedgerErrorExt for E
where
    E: Fail,
{
    fn to_result<D>(self, kind: LedgerErrorKind, msg: D) -> LedgerError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.context(msg).context(kind).into()
    }
}
*/

/*
thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn set_current_error(err: &LedgerError) {
    CURRENT_ERROR_C_JSON
        .try_with(|error| {
            let error_json = json!({
                "message": err.to_string(),
                "backtrace": err.backtrace().map(|bt| bt.to_string())
            })
            .to_string();
            error.replace(Some(string_to_cstring(error_json)));
        })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();
}

pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON
        .try_with(|err| err.borrow().as_ref().map(|err| value = err.as_ptr()))
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();

    value
}

pub fn string_to_cstring(s: String) -> CString {
    CString::new(s).unwrap()
}
*/
