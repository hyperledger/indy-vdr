#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![recursion_limit = "1024"] // for select! macro usage

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "log")]
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
mod utils;

pub mod common;
pub mod config;
#[cfg(feature = "ffi")]
mod ffi;
pub mod ledger;
pub mod pool;
mod state_proof;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn dummy() {
        assert!(true, "Dummy check!");
    }
}
