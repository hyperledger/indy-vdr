use crate::common::error::prelude::*;
use crate::resolver::resolver::PoolRunnerResolver as Resolver;
use std::sync::RwLock;

use std::collections::BTreeMap;

use super::error::{set_last_error, ErrorCode};
use super::pool::{PoolHandle, POOLS};
use crate::ffi::c_char;
use ffi_support::{rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;

new_handle_type!(ResolverHandle, FFI_PH_COUNTER);

static RESOLVERS: Lazy<RwLock<BTreeMap<ResolverHandle, Resolver>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

#[no_mangle]
pub extern "C" fn indy_vdr_resolver_create(pool_handle: usize, handle_p: *mut usize) -> ErrorCode {
    catch_err! {
        trace!("Create resolver");
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        let resolver = Resolver::new(pool);
        let handle = ResolverHandle::next();
        let mut resolvers = write_lock!(RESOLVERS)?;
        resolvers.insert(handle, resolver);
        unsafe {
            *handle_p = *handle;
        }

        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_resolve(
    resolver_handle: usize,
    did: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Resolve DID: {:#?}", did);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();

    let resolvers = read_lock!(RESOLVERS)?;
    let resolver = resolvers.get(&ResolverHandle(resolver_handle))
        .ok_or_else(|| input_err("Unknown resolver handle"))?;

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
            )
            .unwrap();


        Ok(ErrorCode::Success)
    }
}

// #[no_mangle]
// pub extern "C" fn indy_vdr_dereference(
//     resolver_handle: i64,
//     did_url: FfiStr,
//     cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
//     cb_id: i64,
// ) -> ErrorCode {
//     catch_err! {
//         trace!("Dereference DID Url: {:#?}", did_url);
//         let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
//         Ok(ErrorCode::Success)
//     }
// }
