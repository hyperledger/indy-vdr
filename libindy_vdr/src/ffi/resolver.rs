use crate::common::error::prelude::*;
use crate::resolver::{handle_resolution_result, PoolRunnerResolver as Resolver};

use super::error::{set_last_error, ErrorCode};
use super::pool::{PoolHandle, POOLS};
use crate::ffi::c_char;
use ffi_support::{rust_string_to_c, FfiStr};

#[no_mangle]
pub extern "C" fn indy_vdr_resolve(
    pool_handle: PoolHandle,
    did: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
        trace!("Resolve DID: {:#?}", did);
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&pool_handle).ok_or_else(|| input_err("Unknown pool handle"))?;
        let did = did.as_str().to_owned();
        let resolver = Resolver::new(pool);
        resolver
        .dereference(
            did.clone(),
            Box::new(move |ledger_reply| {
                let (errcode, reply) = match handle_resolution_result(ledger_reply, did) {
                    Ok(result) => (ErrorCode::Success, result),
                    Err(err) => {
                        let code = ErrorCode::from(err.kind());
                        set_last_error(Some(err));
                        (code, String::new())
                    }
                };
                cb(cb_id, errcode, rust_string_to_c(reply))
            }),
        )?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_dereference(
    pool_handle: PoolHandle,
    did_url: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
        trace!("Dereference DID Url: {:#?}", did_url);
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&pool_handle).ok_or_else(|| input_err("Unknown pool handle"))?;
        let did_url = did_url.as_str().to_owned();
        let resolver = Resolver::new(pool);
        resolver.dereference(
            did_url.clone(),
            Box::new(move |ledger_reply| {
                let (errcode, reply) = match handle_resolution_result(ledger_reply, did_url) {
                    Ok(result) => (ErrorCode::Success, result),
                    Err(err) => {
                        let code = ErrorCode::from(err.kind());
                        set_last_error(Some(err));
                        (code, String::new())
                    }
                };
                cb(cb_id, errcode, rust_string_to_c(reply))
            }),
        )?;
        Ok(ErrorCode::Success)
    }
}
