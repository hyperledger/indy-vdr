use super::constants::{ATTRIB, GET_ATTR};
use super::did::ShortDidValue;
use super::request::{get_sp_key_marker, RequestType};
use super::response::GetReplyResultV1;
use super::ProtocolVersion;
use crate::common::error::LedgerResult;
use crate::utils::hash::{digest, Sha256};

use named_type::NamedType;

#[derive(Serialize, PartialEq, Debug)]
pub struct AttribOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc: Option<String>,
}

impl AttribOperation {
    pub fn new(
        dest: ShortDidValue,
        hash: Option<String>,
        raw: Option<String>,
        enc: Option<String>,
    ) -> AttribOperation {
        AttribOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
            hash,
            raw,
            enc,
        }
    }
}

impl RequestType for AttribOperation {
    fn get_txn_type<'a>() -> &'a str {
        ATTRIB
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetAttribOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc: Option<String>,
}

impl GetAttribOperation {
    pub fn new(
        dest: ShortDidValue,
        raw: Option<String>,
        hash: Option<String>,
        enc: Option<String>,
    ) -> GetAttribOperation {
        GetAttribOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
            raw,
            hash,
            enc,
        }
    }
}

impl RequestType for GetAttribOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_ATTR
    }

    fn get_sp_key(&self, protocol_version: ProtocolVersion) -> LedgerResult<Option<Vec<u8>>> {
        if let Some(attr_name) = self
            .raw
            .as_ref()
            .or(self.enc.as_ref())
            .or(self.hash.as_ref())
        {
            let marker = get_sp_key_marker(1, protocol_version);
            let hash = digest::<Sha256>(attr_name.as_bytes());
            return Ok(Some(
                format!("{}:{}:{}", self.dest.to_string(), marker, hex::encode(hash))
                    .as_bytes()
                    .to_vec(),
            ));
        }
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetAttrReplyResult {
    GetAttrReplyResultV0(GetAttResultV0),
    GetAttrReplyResultV1(GetReplyResultV1<GetAttResultDataV1>),
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAttResultV0 {
    pub identifier: ShortDidValue,
    pub data: String,
    pub dest: ShortDidValue,
    pub raw: String,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetAttResultDataV1 {
    pub ver: String,
    pub id: String,
    pub did: ShortDidValue,
    pub raw: String,
}

#[derive(Deserialize, Debug)]
pub struct AttribData {
    pub endpoint: Endpoint,
}

#[derive(Serialize, Deserialize, Clone, Debug, NamedType)]
pub struct Endpoint {
    pub ha: String, // indy-node and indy-plenum restrict this to ip-address:port
    pub verkey: Option<String>,
}

impl Endpoint {
    pub fn new(ha: String, verkey: Option<String>) -> Endpoint {
        Endpoint { ha, verkey }
    }
}
