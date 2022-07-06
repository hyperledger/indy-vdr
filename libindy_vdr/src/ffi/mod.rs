use crate::common::error::prelude::*;
use crate::config::{PoolConfig, LIB_VERSION};
use crate::pool::ProtocolVersion;
use crate::utils::Validatable;

use std::convert::TryFrom;
use std::os::raw::c_char;
use std::sync::RwLock;

use ffi_support::{define_string_destructor, rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;

#[macro_use]
mod macros;

mod error;
mod ledger;
mod pool;
mod requests;

use self::error::{set_last_error, ErrorCode};

pub type CallbackId = i64;

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
pub extern "C" fn indy_vdr_set_protocol_version(version: i64) -> ErrorCode {
    catch_err! {
        debug!("Setting pool protocol version: {}", version);
        let version = ProtocolVersion::try_from(version)?;
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
