pub use indy_data_types::anoncreds::rev_reg_def::{
    IssuanceType, RegistryType, RevocationRegistryConfig, RevocationRegistryDefinition,
    RevocationRegistryDefinitionV1, RevocationRegistryDefinitionValue,
    RevocationRegistryDefinitionValuePublicKeys, CL_ACCUM,
};

use super::constants::{GET_REVOC_REG_DEF, REVOC_REG_DEF};
use super::identifiers::CredentialDefinitionId;
use super::identifiers::RevocationRegistryId;
use super::{ProtocolVersion, RequestType};
use crate::common::error::prelude::*;

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
