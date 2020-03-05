use ursa::cl::RevocationKeyPublic;

use super::constants::{GET_REVOC_REG_DEF, REVOC_REG_DEF};
use super::identifiers::cred_def::CredentialDefinitionId;
use super::identifiers::rev_reg::RevocationRegistryId;
use super::{ProtocolVersion, RequestType};
use crate::common::error::prelude::*;
use crate::utils::validation::Validatable;

pub const CL_ACCUM: &str = "CL_ACCUM";

#[derive(Deserialize, Debug, Serialize)]
pub struct RevocationRegistryConfig {
    pub issuance_type: Option<IssuanceType>,
    pub max_cred_num: Option<u32>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum IssuanceType {
    ISSUANCE_BY_DEFAULT,
    ISSUANCE_ON_DEMAND,
}

impl IssuanceType {
    pub fn to_bool(&self) -> bool {
        self.clone() == IssuanceType::ISSUANCE_BY_DEFAULT
    }
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub enum RegistryType {
    CL_ACCUM,
}

impl RegistryType {
    pub fn to_str(&self) -> &'static str {
        match *self {
            RegistryType::CL_ACCUM => CL_ACCUM,
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: IssuanceType,
    pub max_cred_num: u32,
    pub public_keys: RevocationRegistryDefinitionValuePublicKeys,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValuePublicKeys {
    pub accum_key: RevocationKeyPublic,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ver")]
pub enum RevocationRegistryDefinition {
    #[serde(rename = "1.0")]
    RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1),
}

impl RevocationRegistryDefinition {
    pub fn to_unqualified(self) -> RevocationRegistryDefinition {
        match self {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_ref_def) => {
                RevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                    RevocationRegistryDefinitionV1 {
                        id: rev_ref_def.id.to_unqualified(),
                        revoc_def_type: rev_ref_def.revoc_def_type,
                        tag: rev_ref_def.tag,
                        cred_def_id: rev_ref_def.cred_def_id.to_unqualified(),
                        value: rev_ref_def.value,
                    },
                )
            }
        }
    }
}

impl From<RevocationRegistryDefinition> for RevocationRegistryDefinitionV1 {
    fn from(rev_reg_def: RevocationRegistryDefinition) -> Self {
        match rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
        }
    }
}

impl Validatable for RevocationRegistryDefinition {
    fn validate(&self) -> VdrResult<()> {
        match self {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def) => {
                revoc_reg_def.id.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionV1 {
    pub id: RevocationRegistryId,
    pub revoc_def_type: RegistryType,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevRegDefOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RevocationRegistryId,
    #[serde(rename = "revocDefType")]
    pub type_: String,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue,
}

impl RevRegDefOperation {
    pub fn new(rev_reg_def: RevocationRegistryDefinitionV1) -> RevRegDefOperation {
        RevRegDefOperation {
            _type: Self::get_txn_type().to_string(),
            id: rev_reg_def.id,
            type_: rev_reg_def.revoc_def_type.to_str().to_string(),
            tag: rev_reg_def.tag,
            cred_def_id: rev_reg_def.cred_def_id,
            value: rev_reg_def.value,
        }
    }
}

impl RequestType for RevRegDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        REVOC_REG_DEF
    }
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRevRegDefOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RevocationRegistryId,
}

impl GetRevRegDefOperation {
    pub fn new(id: &RevocationRegistryId) -> GetRevRegDefOperation {
        GetRevRegDefOperation {
            _type: Self::get_txn_type().to_string(),
            id: id.clone(),
        }
    }
}

impl RequestType for GetRevRegDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_REVOC_REG_DEF
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        Ok(Some(self.id.as_bytes().to_vec()))
    }
}
