use crate::common::error::prelude::*;
use crate::config::PoolConfig;
use crate::ledger::{PreparedRequest, RequestBuilder};
use crate::pool::{PoolFactory, PoolRunner, ProtocolVersion, RequestResult};
use crate::utils::validation::Validatable;

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::os::raw::c_char;
use std::sync::RwLock;

use serde_json;

use ffi_support::{define_string_destructor, rust_string_to_c, FfiStr};

#[macro_use]
mod macros;
mod error;
use error::{set_last_error, ErrorCode};

define_string_destructor!(indy_vdr_string_free);

new_handle_type!(PoolHandle, FFI_PH_COUNTER);
new_handle_type!(RequestHandle, FFI_RH_COUNTER);

lazy_static! {
    pub static ref POOL_CONFIG: RwLock<PoolConfig> = RwLock::new(PoolConfig::default());
    pub static ref POOLS: RwLock<BTreeMap<PoolHandle, PoolRunner>> = RwLock::new(BTreeMap::new());
    pub static ref REQUESTS: RwLock<BTreeMap<RequestHandle, PreparedRequest>> =
        RwLock::new(BTreeMap::new());
}

#[no_mangle]
pub extern "C" fn indy_vdr_set_protocol_version(version: usize) -> ErrorCode {
    catch_err! {
        debug!("Setting pool protocol version: {}", version);
        let version = ProtocolVersion::try_from(version as u64)?;
        let mut gcfg = write_lock!(POOL_CONFIG)?;
        gcfg.protocol_version = version;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_set_config(config: FfiStr) -> ErrorCode {
    catch_err! {
        trace!("Loading new pool config");
        let config: PoolConfig =
            serde_json::from_str(config.as_str()).with_input_err("Error deserializing config")?;
        config.validate()?;
        debug!("Updating pool config: {:?}", config);
        let mut gcfg = write_lock!(POOL_CONFIG)?;
        *gcfg = config;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_create_from_genesis_file(
    path: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Create pool from genesis file");
        check_useful_c_ptr!(handle_p);
        let mut factory = PoolFactory::from_genesis_file(path.as_str())?;
        {
            let gcfg = read_lock!(POOL_CONFIG)?;
            factory.set_config(*gcfg)?;
        }
        let pool = factory.create_runner()?;
        let handle = PoolHandle::next();
        let mut pools = write_lock!(POOLS)?;
        pools.insert(handle, pool);
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_submit_request(
    pool_handle: usize,
    request_handle: usize,
    cb: Option<extern "C" fn(err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    catch_err! {
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        let req = {
            let mut reqs = write_lock!(REQUESTS)?;
            reqs.remove(&RequestHandle(request_handle))
                .ok_or_else(|| input_err("Unknown request handle"))?
        };
        pool.send_request(req, Box::new(
            move |result| {
                let (errcode, reply) = match result {
                    Ok((reply, _timing)) => {
                        match reply {
                            RequestResult::Reply(body) => {
                                (ErrorCode::Success, body)
                            }
                            RequestResult::Failed(err) => {
                                let code = ErrorCode::from(&err);
                                set_last_error(Some(err));
                                (code, String::new())
                            }
                        }
                    },
                    Err(err) => {
                        let code = ErrorCode::from(&err);
                        set_last_error(Some(err));
                        (code, String::new())
                    }
                };
                cb(errcode, rust_string_to_c(reply))
            }))?;
        Ok(ErrorCode::Success)
    }
}

// NOTE: at the moment, pending requests are allowed to complete
// and request callbacks are still run, even if we no longer have a
// reference to the pool here. Maybe an optional callback for when
// the close has completed?
#[no_mangle]
pub extern "C" fn indy_vdr_pool_close(pool_handle: usize) -> ErrorCode {
    catch_err! {
        let mut pools = write_lock!(POOLS)?;
        pools.remove(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        Ok(ErrorCode::Success)
    }
}

fn get_request_builder() -> LedgerResult<RequestBuilder> {
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
        let (req, _target) = builder.parse_inbound_request_str(request_json.as_str())?;
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
                .with_err_msg(LedgerErrorKind::Unexpected, "Error serializing request body")?
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
pub extern "C" fn indy_vdr_set_default_logger() -> ErrorCode {
    env_logger::init();
    debug!("Initialized default logger");
    ErrorCode::Success
}

/*
- indy_vdr_get_current_error
- indy_vdr_set_protocol_version (for new pools) -> error code
- indy_vdr_set_config (for new pools) -> error code
- indy_vdr_set_logger(callback) -> error code
- indy_vdr_pool_create_from_transactions(char[], *pool_handle) -> error code
- indy_vdr_pool_create_from_genesis_file(char[]) -> error code
- indy_vdr_pool_get_transactions(pool_handle, char[]*) -> error code
- indy_vdr_pool_refresh(pool_handle, callback(command_handle, err, new_txns)) -> error code
- indy_vdr_pool_free(pool_handle) -> void
    (^ no more requests allowed on this pool, but existing ones may be completed)
- indy_vdr_build_{nym, schema, etc}_request(..., *request_handle) -> error code
- indy_vdr_build_custom_request(char[] json, *request_handle) -> error code
- indy_vdr_pool_submit_request(pool_handle, request_handle, callback(command_handle, err, result_json)) -> error code
- indy_vdr_pool_submit_action(pool_handle, request_handle, nodes, timeout, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_free(request_handle) -> void
    (^ only needed for a request that isn't submitted)
- indy_vdr_request_get_body(request_handle, *char[]) -> error code
- indy_vdr_request_get_signature_input(request_handle, *char[]) -> error code
- indy_vdr_request_set_signature(request_handle, *char[]) -> error code
- indy_vdr_request_add_multi_signature(request_handle, *char[]) -> error code
*/
