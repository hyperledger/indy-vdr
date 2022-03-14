use crate::common::error::prelude::*;
use crate::config::{PoolConfig, LIB_VERSION};
use crate::pool::ProtocolVersion;
use crate::utils::Validatable;

use std::convert::TryFrom;
use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::{define_string_destructor, rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;
use serde_json;

#[macro_use]
mod macros;

mod error;
mod ledger;
mod pool;
mod requests;
mod resolver;

use self::error::{set_last_error, ErrorCode};

define_string_destructor!(indy_vdr_string_free);

static POOL_CONFIG: Lazy<RwLock<PoolConfig>> = Lazy::new(|| RwLock::new(PoolConfig::default()));

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
pub extern "C" fn indy_vdr_set_default_logger() -> ErrorCode {
    catch_err! {
        env_logger::init();
        debug!("Initialized default logger");
        Ok(ErrorCode::Success)
    }
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
pub extern "C" fn indy_vdr_set_socks_proxy(socks_proxy: FfiStr) -> ErrorCode {
    catch_err! {
        let proxy = socks_proxy.into_string();
        debug!("Setting pool socks proxy: {}", proxy);
        let mut gcfg = write_lock!(POOL_CONFIG)?;
        gcfg.socks_proxy = Some(proxy);
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_version() -> *mut c_char {
    rust_string_to_c(LIB_VERSION.to_owned())
}

/*
- indy_vdr_get_current_error
- indy_vdr_set_protocol_version (for new pools) -> error code
- indy_vdr_set_config (for new pools) -> error code
- indy_vdr_set_custom_logger(callback) -> error code
- indy_vdr_pool_create_from_transactions(char[], *pool_handle) -> error code
- indy_vdr_pool_create_from_genesis_file(char[]) -> error code
- indy_vdr_pool_get_transactions(pool_handle, char[]*) -> error code
- indy_vdr_pool_refresh(pool_handle, callback(command_handle, err, new_txns)) -> error code
- indy_vdr_pool_close(pool_handle) -> error code
    (^ no more requests allowed on this pool, but existing ones may be completed)
- indy_vdr_build_{nym, schema, etc}_request(..., *request_handle) -> error code
- indy_vdr_build_custom_request(char[] json, *request_handle) -> error code
- indy_vdr_pool_submit_request(pool_handle, request_handle, callback(command_handle, err, result_json)) -> error code
- indy_vdr_pool_submit_action(pool_handle, request_handle, nodes, timeout, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_free(request_handle) -> error code
    (^ only needed for a request that isn't submitted)
- indy_vdr_request_get_body(request_handle, *char[]) -> error code
- indy_vdr_request_get_signature_input(request_handle, *char[]) -> error code
- indy_vdr_request_set_signature(request_handle, *char[]) -> error code
- indy_vdr_request_add_multi_signature(request_handle, *char[]) -> error code
*/
