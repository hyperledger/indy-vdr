use bs58;

use super::error::prelude::*;

pub trait FromBase58 {
    fn from_base58(&self) -> LedgerResult<Vec<u8>>;
}

impl FromBase58 for str {
    fn from_base58(&self) -> LedgerResult<Vec<u8>> {
        bs58::decode(self).into_vec().to_result(
            LedgerErrorKind::InvalidState,
            "Error decoding base58 string",
        )
    }
}
