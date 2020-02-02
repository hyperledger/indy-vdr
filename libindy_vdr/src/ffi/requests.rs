use crate::common::error::prelude::*;
use crate::ledger::{PreparedRequest, RequestBuilder};

use std::collections::BTreeMap;
use std::os::raw::c_char;
use std::sync::RwLock;

use serde_json;

use ffi_support::{rust_string_to_c, FfiStr};

use super::error::{set_last_error, ErrorCode};
use super::POOL_CONFIG;

new_handle_type!(RequestHandle, FFI_RH_COUNTER);

lazy_static! {
    pub static ref REQUESTS: RwLock<BTreeMap<RequestHandle, PreparedRequest>> =
        RwLock::new(BTreeMap::new());
}

fn get_request_builder() -> VdrResult<RequestBuilder> {
    let version = read_lock!(POOL_CONFIG)?.protocol_version;
    Ok(RequestBuilder::new(version))
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_custom_request(
    request_json: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build custom pool request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let (req, _target) = builder.build_custom_request_from_str(request_json.as_str())?;
        let handle = RequestHandle::next();
        let mut requests = write_lock!(REQUESTS)?;
        requests.insert(handle, req);
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_request_get_body(
    request_handle: usize,
    body_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
        trace!("Get request body: {}", request_handle);
        check_useful_c_ptr!(body_p);
        let body = {
            let reqs = read_lock!(REQUESTS)?;
            let req = reqs.get(&RequestHandle(request_handle))
                .ok_or_else(|| input_err("Unknown request handle"))?;
            serde_json::to_string(&req.req_json)
                .with_err_msg(VdrErrorKind::Unexpected, "Error serializing request body")?
        };
        let body = rust_string_to_c(body);
        unsafe {
            *body_p = body;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_request_get_signature_input(
    request_handle: usize,
    input_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
        trace!("Get request signature input: {}", request_handle);
        check_useful_c_ptr!(input_p);
        let sig_input = {
            let reqs = read_lock!(REQUESTS)?;
            let req = reqs.get(&RequestHandle(request_handle))
                .ok_or_else(|| input_err("Unknown request handle"))?;
            req.get_signature_input()?
        };
        let sig_input = rust_string_to_c(sig_input);
        unsafe {
            *input_p = sig_input;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_request_set_signature(
    request_handle: usize,
    signature: *const u8,
    signature_len: usize,
) -> ErrorCode {
    catch_err! {
        trace!("Set request signature: {}", request_handle);
        let signature = slice_from_c_ptr!(signature, signature_len)?;
        let mut reqs = write_lock!(REQUESTS)?;
        let req = reqs.get_mut(&RequestHandle(request_handle))
            .ok_or_else(|| input_err("Unknown request handle"))?;
        req.set_signature(signature)?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_request_free(request_handle: usize) -> ErrorCode {
    catch_err! {
        trace!("Free request: {}", request_handle);
        let mut reqs = write_lock!(REQUESTS)?;
        reqs.remove(&RequestHandle(request_handle))
            .ok_or_else(|| input_err("Unknown request handle"))?;
        Ok(ErrorCode::Success)
    }
}
