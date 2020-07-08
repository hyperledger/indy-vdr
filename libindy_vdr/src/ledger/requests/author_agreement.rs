use std::collections::HashMap;

use crate::common::error::prelude::*;
use crate::utils::{Validatable, ValidationError};

use super::constants::{
    DISABLE_ALL_TXN_AUTHR_AGRMTS, GET_TXN_AUTHR_AGRMT, GET_TXN_AUTHR_AGRMT_AML, TXN_AUTHR_AGRMT,
    TXN_AUTHR_AGRMT_AML,
};
use super::{ProtocolVersion, RequestType};

#[derive(Serialize, PartialEq, Debug)]
pub struct TxnAuthorAgreementOperation {
    #[serde(rename = "type")]
    _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ratification_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retirement_ts: Option<u64>,
}

impl TxnAuthorAgreementOperation {
    pub fn new(
        text: Option<String>,
        version: String,
        ratification_ts: Option<u64>,
        retirement_ts: Option<u64>,
    ) -> TxnAuthorAgreementOperation {
        TxnAuthorAgreementOperation {
            _type: Self::get_txn_type().to_string(),
            text,
            version,
            ratification_ts,
            retirement_ts,
        }
    }
}

impl RequestType for TxnAuthorAgreementOperation {
    fn get_txn_type<'a>() -> &'a str {
        TXN_AUTHR_AGRMT
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct GetTxnAuthorAgreementData {
    pub digest: Option<String>,
    pub version: Option<String>,
    pub timestamp: Option<u64>,
}

impl Validatable for GetTxnAuthorAgreementData {
    fn validate(&self) -> Result<(), ValidationError> {
        match (
            self.digest.as_ref(),
            self.version.as_ref(),
            self.timestamp.as_ref(),
        ) {
            (Some(_), None, None) => Ok(()),
            (None, Some(_), None) => Ok(()),
            (None, None, Some(_)) => Ok(()),
            (None, None, None) => Ok(()),
            (digest, version, timestamp) => {
                Err(invalid!(
                "Only one of field can be specified: digest: {:?}, version: {:?}, timestamp: {:?}",
                digest, version, timestamp
            ))
            }
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetTxnAuthorAgreementOperation {
    #[serde(rename = "type")]
    _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
}

impl GetTxnAuthorAgreementOperation {
    pub fn new(data: Option<&GetTxnAuthorAgreementData>) -> GetTxnAuthorAgreementOperation {
        GetTxnAuthorAgreementOperation {
            _type: Self::get_txn_type().to_string(),
            digest: data.as_ref().and_then(|d| d.digest.clone()),
            version: data.as_ref().and_then(|d| d.version.clone()),
            timestamp: data.as_ref().and_then(|d| d.timestamp),
        }
    }
}

impl RequestType for GetTxnAuthorAgreementOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_TXN_AUTHR_AGRMT
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let key_str = match (
            self.version.as_ref(),
            self.digest.as_ref(),
            self.timestamp.as_ref(),
        ) {
            (None, None, _ts) => "2:latest".to_owned(),
            (None, Some(digest), None) => format!("2:d:{}", digest),
            (Some(version), None, None) => format!("2:v:{}", version),
            _ => return Ok(None),
        };
        Ok(Some(key_str.as_bytes().to_vec()))
    }

    fn get_sp_timestamps(&self) -> VdrResult<(Option<u64>, Option<u64>)> {
        Ok((None, self.timestamp))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AcceptanceMechanisms(pub HashMap<String, ::serde_json::Value>);

impl AcceptanceMechanisms {
    pub fn new() -> Self {
        AcceptanceMechanisms(HashMap::new())
    }
}

impl Validatable for AcceptanceMechanisms {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.0.is_empty() {
            return Err(invalid!(
                "Empty list of Acceptance Mechanisms has been passed",
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetAcceptanceMechanismOperation {
    #[serde(rename = "type")]
    _type: String,
    aml: AcceptanceMechanisms,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aml_context: Option<String>,
}

impl SetAcceptanceMechanismOperation {
    pub fn new(
        aml: AcceptanceMechanisms,
        version: String,
        aml_context: Option<String>,
    ) -> SetAcceptanceMechanismOperation {
        SetAcceptanceMechanismOperation {
            _type: Self::get_txn_type().to_string(),
            aml,
            version,
            aml_context,
        }
    }
}

impl RequestType for SetAcceptanceMechanismOperation {
    fn get_txn_type<'a>() -> &'a str {
        TXN_AUTHR_AGRMT_AML
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetAcceptanceMechanismOperation {
    #[serde(rename = "type")]
    _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl GetAcceptanceMechanismOperation {
    pub fn new(timestamp: Option<u64>, version: Option<String>) -> GetAcceptanceMechanismOperation {
        GetAcceptanceMechanismOperation {
            _type: Self::get_txn_type().to_string(),
            timestamp,
            version,
        }
    }
}

impl RequestType for GetAcceptanceMechanismOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_TXN_AUTHR_AGRMT_AML
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let key_str = if let Some(version) = self.version.as_ref() {
            format!("3:v:{}", version)
        } else {
            "3:latest".to_owned()
        };
        Ok(Some(key_str.as_bytes().to_vec()))
    }

    fn get_sp_timestamps(&self) -> VdrResult<(Option<u64>, Option<u64>)> {
        Ok((None, self.timestamp))
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct DisableAllTxnAuthorAgreementsOperation {
    #[serde(rename = "type")]
    _type: String,
}

impl DisableAllTxnAuthorAgreementsOperation {
    pub fn new() -> Self {
        Self {
            _type: Self::get_txn_type().to_string(),
        }
    }
}

impl RequestType for DisableAllTxnAuthorAgreementsOperation {
    fn get_txn_type<'a>() -> &'a str {
        DISABLE_ALL_TXN_AUTHR_AGRMTS
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthrAgrmtAcceptanceData {
    pub mechanism: String,
    pub taa_digest: String,
    pub time: u64,
}
