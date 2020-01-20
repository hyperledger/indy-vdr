use serde;
use serde_json;
use time;

use crate::domain::did::{DidValue, ShortDidValue};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthrAgrmtAcceptanceData {
    pub mechanism: String,
    pub taa_digest: String,
    pub time: u64,
}

pub fn get_request_id() -> u64 {
    time::get_time().sec as u64 * (1e9 as u64) + time::get_time().nsec as u64
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
    ) -> Result<String, String> {
        // FIXME - verify that qualified DID is using a known DID method

        serde_json::to_string(&Request::new(
            req_id,
            operation,
            identifier.map(DidValue::to_short),
            protocol_version,
        ))
        .map_err(|err| format!("Cannot serialize Request: {:?}", err))
    }
}
