use std::convert::{TryFrom, TryInto};

use crate::common::error::prelude::*;

use super::constants::GET_TXN;
use super::{ProtocolVersion, RequestType};

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
            _type: Self::get_txn_type().to_string(),
            data,
            ledger_id,
        }
    }
}

impl RequestType for GetTxnOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_TXN
    }

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> LedgerResult<Option<Vec<u8>>> {
        Ok(Some(self.data.to_string().into_bytes()))
    }

    fn get_sp_timestamps(&self) -> LedgerResult<(Option<u64>, Option<u64>)> {
        Ok((None, Some(0)))
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
            Self::POOL => Self::POOL as i32,
            Self::DOMAIN => Self::DOMAIN as i32,
            Self::CONFIG => Self::CONFIG as i32,
        }
    }

    pub fn from_id(value: i32) -> LedgerResult<Self> {
        value.try_into()
    }

    pub fn from_str(value: &str) -> LedgerResult<Self> {
        let value = value
            .parse::<i32>()
            .map_err(|_| input_err(format!("Invalid ledger type: {}", value)))?;
        Self::from_id(value)
    }
}

impl TryFrom<i32> for LedgerType {
    type Error = LedgerError;

    fn try_from(value: i32) -> LedgerResult<Self> {
        match value {
            x if x == LedgerType::POOL as i32 => Ok(LedgerType::POOL),
            x if x == LedgerType::DOMAIN as i32 => Ok(LedgerType::DOMAIN),
            x if x == LedgerType::CONFIG as i32 => Ok(LedgerType::CONFIG),
            _ => Err(input_err(format!("Unknown ledger type: {}", value))),
        }
    }
}
