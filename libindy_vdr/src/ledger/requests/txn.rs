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

    fn get_sp_key(&self, _protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        Ok(Some(self.data.to_string().into_bytes()))
    }

    fn get_sp_timestamps(&self) -> VdrResult<(Option<u64>, Option<u64>)> {
        Ok((None, Some(0)))
    }
}
