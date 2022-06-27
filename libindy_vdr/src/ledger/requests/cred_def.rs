pub use indy_data_types::anoncreds::cred_def::{
    CredentialDefinition, CredentialDefinitionData, CredentialDefinitionV1, SignatureType,
    CL_SIGNATURE_TYPE,
};

use super::constants::{CRED_DEF, GET_CRED_DEF};
use super::did::ShortDidValue;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::error::prelude::*;

#[derive(Serialize, Debug)]
pub struct CredDefOperation {
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub data: CredentialDefinitionData,
    #[serde(rename = "type")]
    pub _type: String,
    pub signature_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl CredDefOperation {
    pub fn new(data: CredentialDefinitionV1) -> CredDefOperation {
        CredDefOperation {
            _ref: data.schema_id.0.parse::<i32>().unwrap_or(0),
            signature_type: data.signature_type.to_str().to_string(),
            data: data.value,
            tag: if data.tag.is_empty() {
                None
            } else {
                Some(data.tag.clone())
            },
            _type: Self::get_txn_type().to_string(),
        }
    }
}

impl RequestType for CredDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        CRED_DEF
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetCredDefOperation {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub signature_type: String,
    pub origin: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl GetCredDefOperation {
    pub fn new(
        _ref: i32,
        signature_type: String,
        origin: ShortDidValue,
        tag: Option<String>,
    ) -> GetCredDefOperation {
        GetCredDefOperation {
            _type: Self::get_txn_type().to_string(),
            _ref,
            signature_type,
            origin,
            tag,
        }
    }
}

impl RequestType for GetCredDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_CRED_DEF
    }

    fn get_sp_key(&self, protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let marker = get_sp_key_marker(3, protocol_version);
        let tag = if protocol_version == ProtocolVersion::Node1_3 {
            None
        } else {
            self.tag.clone()
        };
        let tag = tag
            .map(|t| format!(":{}", t))
            .unwrap_or_else(|| "".to_owned());
        Ok(Some(
            format!(
                "{}:{}:{}:{}{}",
                &*self.origin, marker, self.signature_type, self._ref, tag
            )
            .as_bytes()
            .to_vec(),
        ))
    }
}
