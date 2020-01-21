use std::convert::{TryFrom, TryInto};

use serde_json;

use crate::common::error::prelude::*;

use super::constants::GET_TXN;

#[derive(Serialize, PartialEq, Debug)]
pub struct GetTxnOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: i32,
    #[serde(rename = "ledgerId")]
    pub ledger_id: i32,
}

impl GetTxnOperation {
    pub fn new(data: i32, ledger_id: i32) -> GetTxnOperation {
        GetTxnOperation {
            _type: GET_TXN.to_string(),
            data,
            ledger_id,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum LedgerType {
    POOL = 0,
    DOMAIN = 1,
    CONFIG = 2,
}

impl LedgerType {
    pub fn to_id(&self) -> i32 {
        match *self {
            LedgerType::POOL => LedgerType::POOL as i32,
            LedgerType::DOMAIN => LedgerType::DOMAIN as i32,
            LedgerType::CONFIG => LedgerType::CONFIG as i32,
        }
    }

    pub fn from_id(value: i32) -> LedgerResult<Self> {
        value.try_into()
    }

    pub fn from_str(value: &str) -> LedgerResult<Self> {
        serde_json::from_str::<Self>(&format!(r#""{}""#, value)).to_result(
            LedgerErrorKind::InvalidStructure,
            format!("Invalid Ledger type: {}", value),
        )
    }
}

impl TryFrom<i32> for LedgerType {
    type Error = LedgerError;

    fn try_from(value: i32) -> LedgerResult<Self> {
        match value {
            x if x == LedgerType::POOL as i32 => Ok(LedgerType::POOL),
            x if x == LedgerType::DOMAIN as i32 => Ok(LedgerType::DOMAIN),
            x if x == LedgerType::CONFIG as i32 => Ok(LedgerType::CONFIG),
            _ => Err(err_msg(
                LedgerErrorKind::InvalidStructure,
                format!("Invalid Ledger type: {}", value),
            )),
        }
    }
}
