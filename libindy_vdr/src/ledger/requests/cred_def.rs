use super::constants::{CRED_DEF, GET_CRED_DEF};
use super::did::{DidValue, ShortDidValue};
use super::schema::SchemaId;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

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
    fn validate(&self) -> LedgerResult<()> {
        self.id.validate()?;
        self.schema_id.validate()
    }
}

qualifiable_type!(CredentialDefinitionId);

impl CredentialDefinitionId {
    pub const DELIMITER: &'static str = ":";
    pub const PREFIX: &'static str = "creddef";
    pub const MARKER: &'static str = "3";

    pub fn new(
        did: &DidValue,
        schema_id: &SchemaId,
        signature_type: &str,
        tag: &str,
    ) -> CredentialDefinitionId {
        let tag = if tag.is_empty() {
            format!("")
        } else {
            format!("{}{}", Self::DELIMITER, tag)
        };
        let id = CredentialDefinitionId(format!(
            "{}{}{}{}{}{}{}{}",
            did.0,
            Self::DELIMITER,
            Self::MARKER,
            Self::DELIMITER,
            signature_type,
            Self::DELIMITER,
            schema_id.0,
            tag
        ));
        match did.get_method() {
            Some(method) => id.set_method(&method),
            None => id,
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, SchemaId, String)> {
        let parts = self
            .0
            .split_terminator(Self::DELIMITER)
            .collect::<Vec<&str>>();

        if parts.len() == 4 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 5 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = parts[4].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 7 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(Self::DELIMITER);
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 8 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(Self::DELIMITER);
            let tag = parts[7].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 9 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:3:tag
            let did = parts[2..5].join(Self::DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7].to_string();
            let tag = parts[8].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 16 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[2..5].join(Self::DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7..15].join(Self::DELIMITER);
            let tag = parts[15].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        None
    }

    pub fn issuer_did(&self) -> Option<DidValue> {
        self.parts().map(|(did, _, _, _)| did)
    }

    pub fn qualify(&self, method: &str) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => CredentialDefinitionId::new(
                &did.qualify(method),
                &schema_id.qualify(method),
                &signature_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn to_unqualified(&self) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => CredentialDefinitionId::new(
                &did.to_unqualified(),
                &schema_id.to_unqualified(),
                &signature_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn from_str(cred_def_id: &str) -> LedgerResult<Self> {
        let cred_def_id = Self(cred_def_id.to_owned());
        cred_def_id.validate()?;
        Ok(cred_def_id)
    }
}

impl Validatable for CredentialDefinitionId {
    fn validate(&self) -> LedgerResult<()> {
        self.parts().ok_or(input_err(format!(
            "Credential Definition Id validation failed: {:?}, doesn't match pattern",
            self.0
        )))?;
        Ok(())
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

    fn get_sp_key(&self, protocol_version: ProtocolVersion) -> LedgerResult<Option<Vec<u8>>> {
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
