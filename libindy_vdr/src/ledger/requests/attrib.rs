use super::constants::{ATTRIB, GET_ATTR};
use super::did::ShortDidValue;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::error::VdrResult;
use crate::utils::hash::SHA256;

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

    fn get_sp_key(&self, protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        if let Some(attr_name) = self
            .raw
            .as_ref()
            .or(self.enc.as_ref())
            .or(self.hash.as_ref())
        {
            let marker = get_sp_key_marker(1, protocol_version);
            let hash = SHA256::digest(attr_name.as_bytes());
            return Ok(Some(
                format!("{}:{}:{}", &*self.dest, marker, hex::encode(hash))
                    .as_bytes()
                    .to_vec(),
            ));
        }
        Ok(None)
    }
}
