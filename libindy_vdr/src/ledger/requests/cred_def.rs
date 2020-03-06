use super::constants::{CRED_DEF, GET_CRED_DEF};
use super::did::ShortDidValue;
use super::identifiers::cred_def::CredentialDefinitionId;
use super::identifiers::schema::SchemaId;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::error::prelude::*;
use crate::utils::qualifier::Qualifiable;
use crate::utils::validation::{Validatable, ValidationError};

use ursa::cl::{CredentialPrimaryPublicKey, CredentialRevocationPublicKey};

pub const CL_SIGNATURE_TYPE: &str = "CL";

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum SignatureType {
    CL,
}

impl SignatureType {
    pub fn to_str(&self) -> &'static str {
        match *self {
            SignatureType::CL => CL_SIGNATURE_TYPE,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionData {
    pub primary: CredentialPrimaryPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<CredentialRevocationPublicKey>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ver")]
pub enum CredentialDefinition {
    #[serde(rename = "1.0")]
    CredentialDefinitionV1(CredentialDefinitionV1),
}

impl CredentialDefinition {
    pub fn to_unqualified(self) -> CredentialDefinition {
        match self {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => {
                CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
                    id: cred_def.id.to_unqualified(),
                    schema_id: cred_def.schema_id.to_unqualified(),
                    signature_type: cred_def.signature_type,
                    tag: cred_def.tag,
                    value: cred_def.value,
                })
            }
        }
    }
}

impl Validatable for CredentialDefinition {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => cred_def.validate(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDefinitionV1 {
    pub id: CredentialDefinitionId,
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData,
}

impl Validatable for CredentialDefinitionV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        self.id.validate()?;
        self.schema_id.validate()
    }
}

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
                self.origin.to_string(),
                marker,
                self.signature_type,
                self._ref,
                tag
            )
            .as_bytes()
            .to_vec(),
        ))
    }
}

/*
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetCredDefReplyResult {
    GetCredDefReplyResultV0(GetCredDefResultV0),
    GetCredDefReplyResultV1(GetReplyResultV1<GetCredDefResultDataV1>),
}

impl ReplyType for GetCredDefReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_CRED_DEF
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetCredDefResultV0 {
    pub identifier: ShortDidValue,
    #[serde(rename = "ref")]
    pub ref_: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    pub signature_type: SignatureType,
    pub origin: ShortDidValue,
    pub tag: Option<String>,
    pub data: CredentialDefinitionData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCredDefResultDataV1 {
    pub ver: String,
    pub id: CredentialDefinitionId,
    #[serde(rename = "type")]
    pub type_: SignatureType,
    pub tag: String,
    pub schema_ref: SchemaId,
    pub public_keys: CredentialDefinitionData,
}
*/
