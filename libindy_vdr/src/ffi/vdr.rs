use crate::common::error::prelude::*;
use crate::common::handle::ResourceHandle;
use crate::ffi::c_char;
use crate::vdr::RunnerVdr as Vdr;

use ffi_support::{rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;

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
