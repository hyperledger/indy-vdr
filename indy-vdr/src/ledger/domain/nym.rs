use crate::common::error::LedgerResult;
use crate::utils::hash::{digest, Sha256};

use super::constants::{GET_NYM, NYM};
use super::did::ShortDidValue;
use super::request::RequestType;
use super::response::{GetReplyResultV0, GetReplyResultV1, ReplyType};
use super::ProtocolVersion;

#[derive(Serialize, PartialEq, Debug)]
pub struct NymOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<::serde_json::Value>,
}

impl NymOperation {
    pub fn new(
        dest: ShortDidValue,
        verkey: Option<String>,
        alias: Option<String>,
        role: Option<::serde_json::Value>,
    ) -> NymOperation {
        NymOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
            verkey,
            alias,
            role,
        }
    }
}

impl RequestType for NymOperation {
    fn get_txn_type<'a>() -> &'a str {
        NYM
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetNymOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
}

impl GetNymOperation {
    pub fn new(dest: ShortDidValue) -> GetNymOperation {
        GetNymOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
        }
    }
}

impl RequestType for GetNymOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_NYM
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> LedgerResult<Option<Vec<u8>>> {
        let hash = digest::<Sha256>(self.dest.as_bytes());
        Ok(Some(hash))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetNymReplyResult {
    GetNymReplyResultV0(GetReplyResultV0<String>),
    GetNymReplyResultV1(GetReplyResultV1<GetNymResultDataV1>),
}

impl ReplyType for GetNymReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_NYM
    }
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetNymResultDataV0 {
    pub identifier: Option<ShortDidValue>,
    pub dest: ShortDidValue,
    pub role: Option<String>,
    pub verkey: Option<String>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetNymResultDataV1 {
    pub ver: String,
    pub id: String,
    pub did: ShortDidValue,
    pub verkey: Option<String>,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct NymData {
    pub did: ShortDidValue,
    pub verkey: Option<String>,
    pub role: Option<String>,
}
