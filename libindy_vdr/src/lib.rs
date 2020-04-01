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
extern crate vdr_shared;

/// Utility functions, traits and macros
#[macro_use]
pub mod utils;

/// Common data types and error handling
pub mod common;
/// Configuration data types and defaults
pub mod config;
/// Foreign function interface (C API)
#[cfg(feature = "ffi")]
mod ffi;
/// Request and response types for the Indy Node ledger
pub mod ledger;
/// Handling of verifier pool instances and communication
pub mod pool;
/// State proof verification for ledger read transactions
pub mod state_proof;

#[cfg(test)]
#[macro_use]
extern crate rstest;
