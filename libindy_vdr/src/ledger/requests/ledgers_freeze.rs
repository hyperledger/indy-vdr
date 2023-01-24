use super::constants::{GET_FROZEN_LEDGERS, LEDGERS_FREEZE};
use super::RequestType;

#[derive(Serialize, PartialEq, Debug)]
pub struct LedgersFreezeOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub ledgers_ids: Vec<u64>,
}

impl LedgersFreezeOperation {
    pub fn new(ledgers_ids: Vec<u64>) -> LedgersFreezeOperation {
        LedgersFreezeOperation {
            _type: LEDGERS_FREEZE.to_string(),
            ledgers_ids,
        }
    }
}

impl RequestType for LedgersFreezeOperation {
    fn get_txn_type<'a>() -> &'a str {
        LEDGERS_FREEZE
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetFrozenLedgersOperation {
    #[serde(rename = "type")]
    pub _type: String,
}

impl GetFrozenLedgersOperation {
    pub fn new() -> GetFrozenLedgersOperation {
        GetFrozenLedgersOperation {
            _type: GET_FROZEN_LEDGERS.to_string(),
        }
    }
}

impl Default for GetFrozenLedgersOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestType for GetFrozenLedgersOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_FROZEN_LEDGERS
    }
}
