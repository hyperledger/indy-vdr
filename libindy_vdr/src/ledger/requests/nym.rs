use crate::common::error::{input_err, VdrResult};
use crate::utils::hash::SHA256;

use super::constants::{GET_NYM, NYM};
use super::did::ShortDidValue;
use super::{ProtocolVersion, RequestType};
use crate::ledger::constants::{ENDORSER, NETWORK_MONITOR, ROLES, ROLE_REMOVE, STEWARD, TRUSTEE};

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

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let hash = SHA256::digest(self.dest.as_bytes());
        Ok(Some(hash))
    }
}

pub fn role_to_code(role: Option<String>) -> VdrResult<Option<serde_json::Value>> {
    if let Some(r) = role {
        Ok(Some(if r == ROLE_REMOVE {
            serde_json::Value::Null
        } else {
            json!(match r.as_str() {
                "STEWARD" => STEWARD,
                "TRUSTEE" => TRUSTEE,
                "TRUST_ANCHOR" | "ENDORSER" => ENDORSER,
                "NETWORK_MONITOR" => NETWORK_MONITOR,
                role if ROLES.contains(&role) => role,
                role => return Err(input_err(format!("Invalid role: {}", role))),
            })
        }))
    } else {
        Ok(None)
    }
}
