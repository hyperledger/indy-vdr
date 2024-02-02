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
use crate::pool::cache::storage::new_mem_ordered_store;
use crate::pool::cache::{
    storage::{new_fs_ordered_store, OrderedHashMap},
    strategy::CacheStrategyTTL,
};
use crate::pool::{FilesystemCache, PoolTransactionsCache, ProtocolVersion};
use crate::utils::Validatable;

use self::error::{set_last_error, ErrorCode};
use self::pool::{LEDGER_CACHE_STRATEGY, POOL_CACHE, POOL_CONFIG};

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
            debug!("Initializing filesystem pool transactions cache");
            Some(Arc::new(FilesystemCache::new(path)) as Arc<dyn PoolTransactionsCache>)
        } else {
            debug!("Clearing filesystem pool transactions cache");
            None
        };
        *write_lock!(POOL_CACHE)? = cache;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_set_ledger_txn_cache(
    capacity: usize,
    expire_offset: u64,
    path_opt: FfiStr,
) -> ErrorCode {
    catch_err! {
        debug!("Setting pool ledger transactions cache: capacity={}, expire_offset={}", capacity, expire_offset);
        let store = match path_opt.as_opt_str().unwrap_or_default() {
            "" => OrderedHashMap::new(new_mem_ordered_store()),
            path => OrderedHashMap::new(new_fs_ordered_store(path.into())?),
        };

        *write_lock!(LEDGER_CACHE_STRATEGY)? = Some(Arc::new(CacheStrategyTTL::new(capacity, expire_offset.into(), Some(store), None)));
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
