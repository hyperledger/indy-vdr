use crate::common::did::DidValue;
use crate::common::error::prelude::*;

use ffi_support::FfiStr;

use super::error::{set_last_error, ErrorCode};
use super::requests::{add_request, get_request_builder};

#[no_mangle]
pub extern "C" fn indy_vdr_build_custom_request(
    request_json: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build custom pool request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let (req, _target) = builder.build_custom_request_from_str(request_json.as_str(), None, (None, None))?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_txn_request(
    submitter_did: FfiStr, // optional
    ledger_type: i32,
    seq_no: i32,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_TXN request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let req = builder.build_get_txn_request(ledger_type, seq_no, identifier.as_ref())?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_validator_info_request(
    submitter_did: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_VALIDATOR_INFO request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let req = builder.build_get_validator_info_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}
