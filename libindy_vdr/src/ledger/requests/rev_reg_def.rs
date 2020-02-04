use regex::Regex;
use ursa::cl::RevocationKeyPublic;

use super::constants::{GET_REVOC_REG_DEF, REVOC_REG_DEF};
use super::cred_def::CredentialDefinitionId;
use super::{ProtocolVersion, RequestType};
use crate::common::did::DidValue;
use crate::common::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

pub const CL_ACCUM: &str = "CL_ACCUM";

lazy_static! {
    static ref QUALIFIED_REV_REG_ID: Regex = Regex::new("(^revreg:(?P<method>[a-z0-9]+):)?(?P<did>.+):4:(?P<cred_def_id>.+):(?P<rev_reg_type>.+):(?P<tag>.+)$").unwrap();
}

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

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionV1 {
    pub id: RevocationRegistryId,
    pub revoc_def_type: RegistryType,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue,
}

qualifiable_type!(RevocationRegistryId);

impl RevocationRegistryId {
    pub const PREFIX: &'static str = "revreg";
    pub const DELIMITER: &'static str = ":";
    pub const MARKER: &'static str = "3";

    pub fn new(
        did: &DidValue,
        cred_def_id: &CredentialDefinitionId,
        rev_reg_type: &str,
        tag: &str,
    ) -> RevocationRegistryId {
        let id = RevocationRegistryId(format!(
            "{}{}{}{}{}{}{}{}{}",
            did.0,
            Self::DELIMITER,
            Self::MARKER,
            Self::DELIMITER,
            cred_def_id.0,
            Self::DELIMITER,
            rev_reg_type,
            Self::DELIMITER,
            tag
        ));
        match did.get_method() {
            Some(method) => RevocationRegistryId(qualifier::qualify(&id.0, Self::PREFIX, &method)),
            None => id,
        }
    }

    pub fn parts(&self) -> Option<(DidValue, CredentialDefinitionId, String, String)> {
        match QUALIFIED_REV_REG_ID.captures(&self.0) {
            Some(caps) => Some((
                DidValue(caps["did"].to_string()),
                CredentialDefinitionId(caps["cred_def_id"].to_string()),
                caps["rev_reg_type"].to_string(),
                caps["tag"].to_string(),
            )),
            None => None,
        }
    }

    pub fn to_unqualified(&self) -> RevocationRegistryId {
        match self.parts() {
            Some((did, cred_def_id, rev_reg_type, tag)) => RevocationRegistryId::new(
                &did.to_unqualified(),
                &cred_def_id.to_unqualified(),
                &rev_reg_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn from_str(rev_reg_def_id: &str) -> VdrResult<Self> {
        let rev_reg_def_id = Self(rev_reg_def_id.to_owned());
        rev_reg_def_id.validate()?;
        Ok(rev_reg_def_id)
    }
}

impl Validatable for RevocationRegistryId {
    fn validate(&self) -> VdrResult<()> {
        self.parts().ok_or(input_err(format!(
            "Revocation Registry Id validation failed: {:?}, doesn't match pattern",
            self.0
        )))?;
        Ok(())
    }
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
