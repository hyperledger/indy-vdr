use crate::common::error::prelude::*;
use crate::ledger::{RequestBuilder, TxnAuthrAgrmtAcceptanceData};
use crate::pool::PreparedRequest;
use crate::utils::did::DidValue;
use crate::utils::Qualifiable;

use std::collections::BTreeMap;
use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::{rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;

use super::error::{set_last_error, ErrorCode};
use super::POOL_CONFIG;

new_handle_type!(RequestHandle, FFI_RH_COUNTER);

pub static REQUESTS: Lazy<RwLock<BTreeMap<RequestHandle, PreparedRequest>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

pub fn add_request(request: PreparedRequest) -> VdrResult<RequestHandle> {
    let handle = RequestHandle::next();
    let mut requests = write_lock!(REQUESTS)?;
    requests.insert(handle, request);
    Ok(handle)
}

pub fn get_request_builder() -> VdrResult<RequestBuilder> {
    let version = read_lock!(POOL_CONFIG)?.protocol_version;
    Ok(RequestBuilder::new(version))
}

#[no_mangle]
pub extern "C" fn indy_vdr_prepare_txn_author_agreement_acceptance(
    text: FfiStr,       // optional
    version: FfiStr,    // optional
    taa_digest: FfiStr, // optional
    acc_mech_type: FfiStr,
    time: u64,
    output_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
        trace!("Prepare TAA acceptance");
        let builder = get_request_builder()?;
        let acceptance = builder.prepare_txn_author_agreement_acceptance_data(
            text.as_opt_str(),
            version.as_opt_str(),
            taa_digest.as_opt_str(),
            acc_mech_type.as_str(),
            time
        )?;
        let body = rust_string_to_c(serde_json::to_string(&acceptance)
            .with_err_msg(VdrErrorKind::Unexpected, "Error serializing acceptance data")?);
        unsafe {
            *output_p = body;
        }
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
            &req.req_json.to_string()
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
pub extern "C" fn indy_vdr_request_set_endorser(
    request_handle: usize,
    endorser: FfiStr,
) -> ErrorCode {
    catch_err! {
        trace!("Set request endorser: {}", request_handle);
        let endorser = DidValue::from_str(endorser.as_str())?;
        let mut reqs = write_lock!(REQUESTS)?;
        let req = reqs.get_mut(&RequestHandle(request_handle))
            .ok_or_else(|| input_err("Unknown request handle"))?;
        req.set_endorser(&endorser)?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_request_set_multi_signature(
    request_handle: usize,
    identifier: FfiStr,
    signature: *const u8,
    signature_len: usize,
) -> ErrorCode {
    catch_err! {
        trace!("Set request multi signature: {}", request_handle);
        let identifier = DidValue::from_str(identifier.as_str())?;
        let signature = slice_from_c_ptr!(signature, signature_len)?;
        let mut reqs = write_lock!(REQUESTS)?;
        let req = reqs.get_mut(&RequestHandle(request_handle))
            .ok_or_else(|| input_err("Unknown request handle"))?;
        req.set_multi_signature(&identifier, signature)?;
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
pub extern "C" fn indy_vdr_request_set_txn_author_agreement_acceptance(
    request_handle: usize,
    acceptance: FfiStr,
) -> ErrorCode {
    catch_err! {
        trace!("Set request TAA acceptance: {}", request_handle);
        let acceptance = serde_json::from_str::<TxnAuthrAgrmtAcceptanceData>(acceptance.as_str())
            .with_input_err("Invalid TAA acceptance format")?;
        let mut reqs = write_lock!(REQUESTS)?;
        let req = reqs.get_mut(&RequestHandle(request_handle))
            .ok_or_else(|| input_err("Unknown request handle"))?;
        req.set_txn_author_agreement_acceptance(&acceptance)?;
        Ok(ErrorCode::Success)
    }
}
