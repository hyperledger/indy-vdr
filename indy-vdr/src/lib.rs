#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

extern crate hex;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

macro_rules! _map_err {
    ($lvl:expr, $expr:expr) => {
        |err| {
            log!($lvl, "{} - {}", $expr, err);
            err
        }
    };
    ($lvl:expr) => {
        |err| {
            log!($lvl, "{}", err);
            err
        }
    };
}

#[macro_export]
macro_rules! map_err_err {
    () => ( _map_err!(::log::Level::Error) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Error, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_trace {
    () => ( _map_err!(::log::Level::Trace) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Trace, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_info {
    () => ( _map_err!(::log::Level::Info) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Info, $($arg)*) )
}

macro_rules! unwrap_opt_or_return {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(val) => val,
            None => return $err,
        };
    };
}

macro_rules! unwrap_or_return {
    ($result:expr, $err:expr) => {
        match $result {
            Ok(res) => res,
            Err(_) => return $err,
        };
    };
}

#[macro_use]
mod utils;

pub mod api;
pub mod common;
pub mod config;
pub mod ledger;
pub mod pool;
pub mod state_proof;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn dummy() {
        assert!(true, "Dummy check!");
    }
}
