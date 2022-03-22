use crate::common::error::prelude::*;
use crate::common::handle::ResourceHandle;
use crate::ffi::c_char;
use crate::pool::{PoolBuilder, PoolTransactions};
use crate::vdr::RunnerVdr as Vdr;

use super::POOL_CONFIG;

use ffi_support::{rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;
use std::thread;

use super::error::{set_last_error, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VdrHandle(pub i64);

impl_sequence_handle!(VdrHandle, FFI_PH_COUNTER);

pub static VDRS: Lazy<RwLock<BTreeMap<VdrHandle, Vdr>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

#[cfg(feature = "git")]
#[no_mangle]
pub extern "C" fn indy_vdr_create_from_github(handle_p: *mut VdrHandle) -> ErrorCode {
    catch_err! {
        trace!("Create VDR from github");
        check_useful_c_ptr!(handle_p);

        let vdr = Vdr::from_github(None)?;
        let handle = VdrHandle::next();
        let mut vdrs = write_lock!(VDRS)?;
        vdrs.insert(handle, vdr);
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_create_from_folder(
    path: FfiStr,
    genesis_filename: FfiStr, // Optional
    handle_p: *mut VdrHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Create VDR from folder");
        check_useful_c_ptr!(handle_p);

        let path = PathBuf::from(path.as_str());
        let genesis_filename = genesis_filename.as_opt_str();

        let vdr = Vdr::from_folder(path, genesis_filename)?;
        let handle = VdrHandle::next();
        let mut vdrs = write_lock!(VDRS)?;
        vdrs.insert(handle, vdr);
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub unsafe extern "C" fn indy_vdr_resolve(
    vdr_handle: VdrHandle,
    did: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Resolve DID: {:#?}", did);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();
    let vdrs = read_lock!(VDRS)?;
    let vdr = vdrs.get(&vdr_handle)
        .ok_or_else(|| input_err("Unknown VDR handle"))?;
    vdr.resolve(did.as_str(), Box::new(move |result| {
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
pub unsafe extern "C" fn indy_vdr_dereference(
    vdr_handle: VdrHandle,
    did_url: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode, response: *const c_char)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Dereference DID: {:#?}", did_url);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();
    let vdrs = read_lock!(VDRS)?;
    let vdr = vdrs.get(&vdr_handle)
        .ok_or_else(|| input_err("Unknown VDR handle"))?;
    vdr.dereference(did_url.as_str(), Box::new(move |result| {
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

#[allow(unused_variables)]
fn handle_pool_refresh(
    vdr_handle: VdrHandle,
    ledger: String,
    old_txns: Vec<String>,
    new_txns: Vec<String>,
) -> ErrorCode {
    catch_err!(
        debug!("Adding {} new pool transactions", new_txns.len());
        let mut txns = PoolTransactions::from_json_transactions(old_txns)?;
        txns.extend_from_json(&new_txns)?;
        let builder = {
            let gcfg = read_lock!(POOL_CONFIG)?;
            PoolBuilder::from(gcfg.clone())
        };
        let pool = builder.transactions(txns)?.into_runner()?;
        let vdrs = read_lock!(VDRS).unwrap();
        // FIXME: Can't get mutable reference
        // let vdr = vdrs.get_mut(&vdr_handle)
            // .ok_or_else(|| input_err("Unknown VDR handle"))?;
        // vdr.set_pool(ledger, pool);
        Ok(ErrorCode::Success)
    )
}

#[no_mangle]
pub unsafe extern "C" fn indy_vdr_refresh(
    vdr_handle: VdrHandle,
    ledger: FfiStr,
    cb: Option<extern "C" fn(cb_id: i64, err: ErrorCode)>,
    cb_id: i64,
) -> ErrorCode {
    catch_err! {
    trace!("Refresh ledger: {:#?}", ledger);
    let cb = cb.ok_or_else(|| input_err("No callback provided")).unwrap();
    let vdrs = read_lock!(VDRS)?;
    let vdr = vdrs.get(&vdr_handle)
        .ok_or_else(|| input_err("Unknown VDR handle"))?;
    let ledger = ledger.as_str();
    let ledger = ledger.to_string();
    let pool = vdr.get_pool(&ledger);
    if let Some(pool) = pool {


    pool.refresh(Box::new(
        move |result| {
            let errcode = match result {
                Ok((old_txns, new_txns, _timing)) => {
                    if let Some(new_txns) = new_txns {
                        // We must spawn a new thread here because this callback
                        // is being run in the PoolRunner's thread, and if we drop
                        // the instance now it will create a deadlock
                        thread::spawn(move || {
                            let result = handle_pool_refresh(vdr_handle, ledger, old_txns, new_txns);
                            cb(cb_id, result)
                        });
                        return
                    } else {
                        ErrorCode::Success
                    }
                },
                Err(err) => {
                    let code = ErrorCode::from(err.kind());
                    set_last_error(Some(err));
                    code
                }
            };
            cb(cb_id, errcode)
        }))?;

    }
    Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_get_ledgers(
    vdr_handle: VdrHandle,
    handle_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
    trace!("VDR Get Ledgers");
    let vdrs = read_lock!(VDRS)?;
    let vdr = vdrs.get(&vdr_handle)
        .ok_or_else(|| input_err("Unknown VDR handle"))?;

    let ledgers = rust_string_to_c(vdr.get_ledgers()?.join(";"));
    unsafe {
        *handle_p = ledgers;

    }
    Ok(ErrorCode::Success)
    }
}
