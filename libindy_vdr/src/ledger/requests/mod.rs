/// ATTRIB, GET_ATTRIB transaction operations
pub mod attrib;
/// AUTH_RULE and related transaction operations
pub mod auth_rule;
/// Transaction author agreement data types and operations
pub mod author_agreement;
/// Credential definition operations
pub mod cred_def;
/// Frozen Ledger operations
pub mod ledgers_freeze;
/// NODE transactions operations
pub mod node;
/// NYM transaction operations
pub mod nym;
/// Verifier pool configuration and upgrade operations
pub mod pool;
/// Revocation registry operations
pub mod rev_reg;
/// Revocation registry definition operations
pub mod rev_reg_def;
#[cfg(any(feature = "rich_schema", test))]
/// Rich schema operations
#[macro_use]
pub mod rich_schema;
/// V1 schema operations
pub mod schema;
/// GET_TXN operation
pub mod txn;
/// GET_VALIDATOR_INFO operation
pub mod validator_info;

use std::collections::HashMap;

use serde;
use serde_json;

use super::constants;
use super::identifiers;
use crate::common::error::prelude::*;
use crate::pool::ProtocolVersion;
use crate::utils::did::{self, DidValue, ShortDidValue};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Request<T: serde::Serialize> {
    pub req_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<ShortDidValue>,
    pub operation: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taa_acceptance: Option<author_agreement::TxnAuthrAgrmtAcceptanceData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endorser: Option<ShortDidValue>,
}

impl<T: serde::Serialize> Request<T> {
    pub fn new(
        req_id: i64,
        operation: T,
        identifier: Option<ShortDidValue>,
        protocol_version: Option<i64>,
    ) -> Request<T> {
        Request {
            req_id,
            identifier,
            operation,
            protocol_version,
            signature: None,
            signatures: None,
            taa_acceptance: None,
            endorser: None,
        }
    }

    pub fn build_request(
        req_id: i64,
        operation: T,
        identifier: Option<&DidValue>,
        protocol_version: Option<i64>,
    ) -> VdrResult<serde_json::Value> {
        // FIXME - verify that qualified DID is using a known DID method

        serde_json::to_value(&Request::new(
            req_id,
            operation,
            identifier.map(DidValue::to_short),
            protocol_version,
        ))
        .with_input_err("Cannot serialize request")
    }
}

/// Base trait for all ledger transaction request operations
pub trait RequestType: serde::Serialize {
    /// Get the transaction type as a numeric string
    fn get_txn_type<'a>() -> &'a str;

    /// Get a state proof key for the transaction, if any can be derived
    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Get the state proof timestamps for the request, if any
    fn get_sp_timestamps(&self) -> VdrResult<(Option<u64>, Option<u64>)> {
        Ok((None, None))
    }
}

/// Format a transaction type marker according to the provided protocol version
pub fn get_sp_key_marker(code: u8, protocol_version: ProtocolVersion) -> char {
    if protocol_version == ProtocolVersion::Node1_3 {
        code as char
    } else {
        (code + 48) as char // digit as ascii
    }
}
