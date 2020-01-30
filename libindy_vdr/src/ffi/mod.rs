use crate::common::error::prelude::*;
use crate::config::PoolConfig;
use crate::ledger::PreparedRequest;
use crate::pool::{Pool, PoolFactory, ProtocolVersion, SharedPool};
use crate::utils::validation::Validatable;

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::os::raw::c_char;
use std::sync::RwLock;

use serde_json;

use ffi_support::{define_string_destructor, rust_string_to_c, FfiStr};

define_string_destructor!(indy_vdr_string_free);

new_handle_type!(PoolHandle, FFI_PH_COUNTER);
new_handle_type!(RequestHandle, FFI_RH_COUNTER);

macro_rules! catch_err {
    ($($e:tt)*) => {
        match (|| -> LedgerResult<_> {$($e)*})() {
            Ok(a) => a,
            // FIXME - save last error for retrieval
            Err(ref err) => return ErrorCode::from(err),
        }
    }
}
macro_rules! read_lock {
    ($e:expr) => {
        ($e).read()
            .map_err(|_| err_msg(LedgerErrorKind::Unexpected, "Error acquiring read lock"))
    };
}
macro_rules! write_lock {
    ($e:expr) => {
        ($e).write()
            .map_err(|_| err_msg(LedgerErrorKind::Unexpected, "Error acquiring write lock"))
    };
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(usize)]
pub enum ErrorCode {
    Success = 0,
    Failed = 1,
}

impl From<&LedgerError> for ErrorCode {
    fn from(_err: &LedgerError) -> ErrorCode {
        ErrorCode::Failed
    }
}

impl<T> From<LedgerResult<T>> for ErrorCode {
    fn from(result: LedgerResult<T>) -> ErrorCode {
        match result {
            Ok(_) => ErrorCode::Success,
            Err(ref err) => ErrorCode::from(err),
        }
    }
}

lazy_static! {
    pub static ref POOL_CONFIG: RwLock<PoolConfig> = RwLock::new(PoolConfig::default());
    pub static ref POOLS: RwLock<BTreeMap<PoolHandle, SharedPool>> = RwLock::new(BTreeMap::new());
    pub static ref REQUESTS: RwLock<BTreeMap<RequestHandle, PreparedRequest>> =
        RwLock::new(BTreeMap::new());
}

#[no_mangle]
extern "C" fn indy_vdr_set_protocol_version(version: usize) -> ErrorCode {
    catch_err! {
        let version = ProtocolVersion::try_from(version as u64)?;
        let mut gcfg = write_lock!(POOL_CONFIG)?;
        gcfg.protocol_version = version;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
extern "C" fn indy_vdr_set_config(config: FfiStr) -> ErrorCode {
    catch_err! {
        let config: PoolConfig =
            serde_json::from_str(config.as_str()).with_input_err("Error deserializing config")?;
        config.validate()?;
        let mut gcfg = write_lock!(POOL_CONFIG)?;
        *gcfg = config;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
extern "C" fn indy_vdr_pool_create_from_genesis_file(
    path: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        // check_useful_c_ptr!(handle_p)
        let mut factory = PoolFactory::from_genesis_file(path.as_str())?;
        {
            let gcfg = read_lock!(POOL_CONFIG)?;
            factory.set_config(*gcfg)?;
        }
        let pool = factory.create_shared()?;
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
extern "C" fn indy_vdr_pool_build_custom_request(
    pool_handle: usize,
    request_json: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        // check_useful_c_ptr!(handle_p)
        let builder = {
            let pools = read_lock!(POOLS)?;
            let pool = pools.get(&PoolHandle(pool_handle))
                .ok_or_else(|| err_msg(LedgerErrorKind::Input, "Unknown pool handle"))?;
            pool.get_request_builder()
        };
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
extern "C" fn indy_vdr_request_get_body(
    request_handle: usize,
    body_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
        // check_useful_c_ptr!(body_p)
        let body = {
            let reqs = read_lock!(REQUESTS)?;
            let req = reqs.get(&RequestHandle(request_handle))
                .ok_or_else(|| err_msg(LedgerErrorKind::Input, "Unknown pool handle"))?;
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
extern "C" fn indy_vdr_request_get_signature_input(
    request_handle: usize,
    input_p: *mut *const c_char,
) -> ErrorCode {
    catch_err! {
        // check_useful_c_ptr!(input_p)
        let sig_input = {
            let reqs = read_lock!(REQUESTS)?;
            let req = reqs.get(&RequestHandle(request_handle))
                .ok_or_else(|| err_msg(LedgerErrorKind::Input, "Unknown pool handle"))?;
            req.get_signature_input()?
        };
        let sig_input = rust_string_to_c(sig_input);
        unsafe {
            *input_p = sig_input;
        }
        Ok(ErrorCode::Success)
    }
}

/*
- indy_vdr_get_last_error
- indy_vdr_set_protocol_version (for new pools) -> error code
- indy_vdr_set_config (for new pools) -> error code
- indy_vdr_set_logger(callback) -> error code
- indy_vdr_pool_create_from_transactions(char[], *pool_handle) -> error code
- indy_vdr_pool_create_from_genesis_file(char[]) -> error code
- indy_vdr_pool_get_transactions(pool_handle, char[]*) -> error code
- indy_vdr_pool_refresh(pool_handle, callback(command_handle, err, new_txns)) -> error code
- indy_vdr_pool_free(pool_handle) -> void
    (^ no more requests allowed on this pool, but existing ones may be completed)
- indy_vdr_pool_build_{nym, schema, etc}_request(pool_handle, ..., *request_handle) -> error code
- indy_vdr_pool_build_custom_request(pool_handle, char[] json, *request_handle) -> error code
- indy_vdr_request_submit(request_handle, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_submit_action(request_handle, nodes, timeout, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_free(request_handle) -> void
    (^ only needed for a request that isn't submitted)
- indy_vdr_request_get_body(request_handle, *char[]) -> error code
- indy_vdr_request_get_signature_input(request_handle, *char[]) -> error code
- indy_vdr_request_set_signature(request_handle, *char[]) -> error code
- indy_vdr_request_add_multi_signature(request_handle, *char[]) -> error code
*/
