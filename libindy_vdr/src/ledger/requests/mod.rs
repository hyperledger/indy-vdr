pub mod attrib;
pub mod auth_rule;
pub mod author_agreement;
pub mod cred_def;
pub mod node;
pub mod nym;
pub mod pool;
pub mod rev_reg;
pub mod rev_reg_def;
pub mod rich_schema;
pub mod schema;
pub mod txn;
pub mod validator_info;

pub use super::constants;
pub use super::identifiers;
pub use crate::common::did;
pub use crate::common::verkey;
pub use crate::pool::ProtocolVersion;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde;
use serde_json;

use crate::common::error::prelude::*;
use did::{DidValue, ShortDidValue};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthrAgrmtAcceptanceData {
    pub mechanism: String,
    pub taa_digest: String,
    pub time: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<T: serde::Serialize> {
    pub req_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<ShortDidValue>,
    pub operation: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taa_acceptance: Option<TxnAuthrAgrmtAcceptanceData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endorser: Option<ShortDidValue>,
}

impl<T: serde::Serialize> Request<T> {
    pub fn new(
        req_id: u64,
        operation: T,
        identifier: Option<ShortDidValue>,
        protocol_version: Option<usize>,
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
        req_id: u64,
        operation: T,
        identifier: Option<&DidValue>,
        protocol_version: Option<usize>,
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

pub trait RequestType: serde::Serialize {
    fn get_txn_type<'a>() -> &'a str;

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_sp_timestamps(&self) -> VdrResult<(Option<u64>, Option<u64>)> {
        Ok((None, None))
    }
}

pub fn get_sp_key_marker(code: u8, protocol_version: ProtocolVersion) -> char {
    if protocol_version == ProtocolVersion::Node1_3 {
        code as char
    } else {
        (code + 48) as char // digit as ascii
    }
}

pub fn get_request_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards")
        .as_nanos() as u64
}
