use crate::common::error::prelude::*;

use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::rust_string_to_c;

lazy_static! {
    pub static ref LAST_ERROR: RwLock<Option<VdrError>> = RwLock::new(None);
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
#[repr(usize)]
pub enum ErrorCode {
    Success = 0,
    Failed = 1,
}

impl From<&VdrError> for ErrorCode {
    fn from(_err: &VdrError) -> ErrorCode {
        ErrorCode::Failed
    }
}

impl<T> From<VdrResult<T>> for ErrorCode {
    fn from(result: VdrResult<T>) -> ErrorCode {
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
