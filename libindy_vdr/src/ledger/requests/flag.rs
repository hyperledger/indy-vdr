use crate::common::error::VdrResult;
use crate::ledger::requests::get_sp_key_marker;

use super::constants::{FLAG, GET_FLAG};
use super::{ProtocolVersion, RequestType};

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlagOperation {
    pub _type: String,
    pub name: String,
    pub value: String,
}

impl FlagOperation {
    pub fn new(name: String, value: String) -> FlagOperation {
        FlagOperation {
            _type: Self::get_txn_type().to_string(),
            name,
            value,
        }
    }
}

impl RequestType for FlagOperation {
    fn get_txn_type<'a>() -> &'a str {
        FLAG
    }
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFlagOperation {
    pub _type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl GetFlagOperation {
    pub fn new(name: String, seq_no: Option<i32>, timestamp: Option<u64>) -> GetFlagOperation {
        GetFlagOperation {
            _type: Self::get_txn_type().to_string(),
            name,
            seq_no,
            timestamp,
        }
    }
}

impl RequestType for GetFlagOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_FLAG
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        if !self.name.is_empty() {
            let marker = get_sp_key_marker(2, _protocol_version);
            return Ok(Some(
                format!("{}:{}", marker, self.name).as_bytes().to_vec(),
            ));
        }
        Ok(None)
    }
}
