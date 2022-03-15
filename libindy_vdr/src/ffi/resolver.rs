use crate::common::error::prelude::*;
use crate::resolver::resolver::PoolRunnerResolver as Resolver;

use super::error::{set_last_error, ErrorCode};
use super::pool::{PoolHandle, POOLS};
use crate::ffi::c_char;
use ffi_support::{rust_string_to_c, FfiStr};


#[no_mangle]
pub extern "C" fn indy_vdr_resolve(
    pool_handle: usize,
    did: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Resolve DID: {:#?}", did);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();
    let pools = read_lock!(POOLS)?;
    let pool = pools.get(&PoolHandle(pool_handle)).ok_or_else(|| input_err("Unknown pool handle"))?;
    let resolver = Resolver::new(pool);
        resolver
            .resolve(
                did.as_str(),
                Box::new(move |result| {
                    let (errcode, reply) = match result {
                        Ok(status) => (ErrorCode::Success, status),
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
    pool_handle: usize,
    did_url: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Dereference DID Url: {:#?}", did_url);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();
    let pools = read_lock!(POOLS)?;
    let pool = pools.get(&PoolHandle(pool_handle)).ok_or_else(|| input_err("Unknown pool handle"))?;
    let resolver =Resolver::new(pool);
        resolver
            .dereference(
                did_url.as_str(),
                Box::new(move |result| {
                    let (errcode, reply) = match result {
                        Ok(status) => (ErrorCode::Success, status),
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
