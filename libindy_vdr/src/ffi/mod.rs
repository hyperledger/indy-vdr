use std::convert::TryFrom;
use std::os::raw::c_char;
use std::sync::Arc;

use ffi_support::{define_string_destructor, rust_string_to_c, FfiStr};

#[macro_use]
mod macros;

mod error;
mod ledger;
mod pool;
mod requests;
mod resolver;

use crate::common::error::prelude::*;
use crate::config::{PoolConfig, LIB_VERSION};
use crate::pool::ProtocolVersion;
use crate::pool::{FilesystemCache, PoolTransactionsCache};
use crate::utils::Validatable;

use self::error::{set_last_error, ErrorCode};
use self::pool::{POOL_CACHE, POOL_CONFIG};

pub type CallbackId = i64;

define_string_destructor!(indy_vdr_string_free);

#[no_mangle]
pub extern "C" fn indy_vdr_set_config(config: FfiStr) -> ErrorCode {
    catch_err! {
        trace!("Loading new pool config");
        let config: PoolConfig =
            serde_json::from_str(config.as_str()).with_input_err("Error deserializing config")?;
        config.validate()?;
        debug!("Updating pool config: {:?}", config);
        *write_lock!(POOL_CONFIG)? = config;
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
pub extern "C" fn indy_vdr_set_protocol_version(version: i64) -> ErrorCode {
    catch_err! {
        debug!("Setting pool protocol version: {}", version);
        let version = ProtocolVersion::try_from(version)?;
        write_lock!(POOL_CONFIG)?.protocol_version = version;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_set_cache_directory(path: FfiStr) -> ErrorCode {
    catch_err! {
        let cache = if let Some(path) = path.as_opt_str() {
            trace!("Initializing filesystem pool transactions cache");
            Some(Arc::new(FilesystemCache::new(path)) as Arc<dyn PoolTransactionsCache>)
        } else {
            trace!("Clearing filesystem pool transactions cache");
            None
        };
        *write_lock!(POOL_CACHE)? = cache;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_set_socks_proxy(socks_proxy: FfiStr) -> ErrorCode {
    catch_err! {
        let proxy = socks_proxy.into_string();
        debug!("Setting pool socks proxy: {}", proxy);
        write_lock!(POOL_CONFIG)?.socks_proxy.replace(proxy);
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_version() -> *mut c_char {
    rust_string_to_c(LIB_VERSION.to_owned())
}
