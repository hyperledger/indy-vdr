use crate::common::error::prelude::*;

use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::rust_string_to_c;

lazy_static! {
    pub static ref LAST_ERROR: RwLock<Option<LedgerError>> = RwLock::new(None);
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
#[repr(usize)]
pub enum ErrorCode {
    Success = 0,
    Failed = 1,
}

impl From<&LedgerError> for ErrorCode {
    fn from(_err: &LedgerError) -> ErrorCode {
        ErrorCode::Failed
    }
}

impl<T> From<LedgerResult<T>> for ErrorCode {
    fn from(result: LedgerResult<T>) -> ErrorCode {
        match result {
            Ok(_) => ErrorCode::Success,
            Err(ref err) => ErrorCode::from(err),
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
        let code = ErrorCode::from(&err) as usize;
        json!({"code": code, "message": message}).to_string()
    } else {
        "{\"code\": 0, \"message\": \"\"}".to_owned()
    }
}

pub fn set_last_error(error: Option<LedgerError>) {
    trace!("indy_vdr_set_last_error");
    *LAST_ERROR.write().unwrap() = error;
}
