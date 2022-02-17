//! A library for interacting with Hyperledger Indy Node ledger instances
//!
//! Indy-VDR provides classes for connecting to a ledger, preparing requests for
//! the validator nodes, and collecting the results. For alternative use cases
//! try the `indy-vdr-proxy` standalone webserver for a basic HTTP interface, the
//! C API (FFI), or the Python wrapper.
//!
//! # Getting Started
//!
//! As a basic example, the code below demonstrates creating a [`pool::LocalPool`] instance
//! and performing a transaction read request. There are additional helper functions
//! in the [`pool::helpers`] module but in most cases you will use a [`ledger::RequestBuilder`]
//! to construct a [`pool::PreparedRequest`] and dispatch it.
//!
//! ```no_run
//! use futures_executor::block_on;
//! use indy_vdr::pool::{
//!     helpers::perform_get_txn,
//!     PoolBuilder,
//!     PoolTransactions
//! };
//!
//! // Load genesis transactions. The corresponding transactions for the ledger you
//! // are connecting to should be saved to a local file.
//! let txns = PoolTransactions::from_json_file("./genesis.txn").unwrap();
//!
//! // Create a PoolBuilder instance
//! let pool_builder = PoolBuilder::default().transactions(txns).unwrap();
//! // Convert into a thread-local Pool instance
//! let pool = pool_builder.into_local().unwrap();
//!
//! // Create a new GET_TXN request and dispatch it
//! let ledger_type = 1;  // 1 identifies the Domain ledger, see pool::LedgerType
//! let seq_no = 1;       // Transaction sequence number
//! let (result, _timing) = block_on(perform_get_txn(&pool, ledger_type, seq_no)).unwrap();

#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![recursion_limit = "1024"] // for select! macro usage

#[cfg(feature = "log")]
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate indy_utils;

/// Utility functions, traits and macros
#[macro_use]
pub mod utils;

/// Common data types and error handling
#[macro_use]
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
