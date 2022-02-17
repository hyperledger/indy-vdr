use crate::common::error::prelude::*;

use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::rust_string_to_c;
use once_cell::sync::Lazy;

pub static LAST_ERROR: Lazy<RwLock<Option<VdrError>>> = Lazy::new(|| RwLock::new(None));

#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
#[repr(i64)]
pub enum ErrorCode {
    Success = 0,
    Config = 1,
    Connection = 2,
    FileSystem = 3,
    Input = 4,
    Resource = 5,
    Unavailable = 6,
    Unexpected = 7,
    Incompatible = 8,
    PoolNoConsensus = 30,
    PoolRequestFailed = 31,
    PoolTimeout = 32,
}

impl From<&VdrErrorKind> for ErrorCode {
    fn from(kind: &VdrErrorKind) -> ErrorCode {
        match kind {
            VdrErrorKind::Config => ErrorCode::Config,
            VdrErrorKind::Connection => ErrorCode::Connection,
            VdrErrorKind::FileSystem(_) => ErrorCode::FileSystem,
            VdrErrorKind::Input => ErrorCode::Input,
            VdrErrorKind::Resource => ErrorCode::Resource,
            VdrErrorKind::Unavailable => ErrorCode::Unavailable,
            VdrErrorKind::Unexpected => ErrorCode::Unexpected,
            VdrErrorKind::Incompatible => ErrorCode::Incompatible,
            VdrErrorKind::PoolNoConsensus => ErrorCode::PoolNoConsensus,
            VdrErrorKind::PoolRequestFailed(_) => ErrorCode::PoolRequestFailed,
            VdrErrorKind::PoolTimeout => ErrorCode::PoolTimeout,
        }
    }
}

impl<T> From<VdrResult<T>> for ErrorCode {
    fn from(result: VdrResult<T>) -> ErrorCode {
        match result {
            Ok(_) => ErrorCode::Success,
            Err(err) => ErrorCode::from(err.kind()),
        }
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_get_current_error(error_json_p: *mut *const c_char) -> ErrorCode {
    trace!("indy_vdr_get_current_error");

    let error = rust_string_to_c(get_current_error_json());
    unsafe { *error_json_p = error };

    ErrorCode::Success
}

pub fn get_current_error_json() -> String {
    if let Some(err) = LAST_ERROR.write().unwrap().take() {
        let message = err.to_string();
        let code = ErrorCode::from(err.kind()) as i64;
        let extra = err.extra();
        json!({"code": code, "message": message, "extra": extra}).to_string()
    } else {
        r#"{"code":0,"message":null,"extra":null}"#.to_owned()
    }
}

pub fn set_last_error(error: Option<VdrError>) {
    trace!("indy_vdr_set_last_error");
    *LAST_ERROR.write().unwrap() = error;
}
