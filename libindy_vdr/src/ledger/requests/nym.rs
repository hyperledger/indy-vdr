use sha2::{Digest, Sha256};

use super::constants::{GET_NYM, NYM};
use super::did::ShortDidValue;
use super::{ProtocolVersion, RequestType};
use crate::common::error::VdrResult;
use crate::ledger::constants::UpdateRole;

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NymOperation {
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UpdateRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diddoc_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

impl NymOperation {
    pub fn new(
        dest: ShortDidValue,
        verkey: Option<String>,
        alias: Option<String>,
        role: Option<UpdateRole>,
        diddoc_content: Option<String>,
        version: Option<i32>,
    ) -> NymOperation {
        NymOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
            verkey,
            alias,
            role,
            diddoc_content,
            version,
        }
    }
}

impl RequestType for NymOperation {
    fn get_txn_type<'a>() -> &'a str {
        NYM
    }
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNymOperation {
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl GetNymOperation {
    pub fn new(
        dest: ShortDidValue,
        seq_no: Option<i32>,
        timestamp: Option<u64>,
    ) -> GetNymOperation {
        GetNymOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
            seq_no,
            timestamp,
        }
    }
}

impl RequestType for GetNymOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_NYM
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let hash = Sha256::digest(self.dest.as_bytes()).to_vec();
        Ok(Some(hash))
    }
}
