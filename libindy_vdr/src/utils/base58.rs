use bs58;

use super::validation::ValidationError;

pub trait FromBase58 {
    fn from_base58(&self) -> Result<Vec<u8>, ValidationError>;
}

impl FromBase58 for str {
    fn from_base58(&self) -> Result<Vec<u8>, ValidationError> {
        bs58::decode(self)
            .into_vec()
            .map_err(|_| ValidationError(Some("Error decoding base58 string".to_owned())))
    }
}

pub fn decode<T: FromBase58>(val: &T) -> Result<Vec<u8>, ValidationError> {
    val.from_base58()
}

pub trait ToBase58 {
    fn to_base58(&self) -> String;
}

impl ToBase58 for [u8] {
    fn to_base58(&self) -> String {
        bs58::encode(self).into_string()
    }
}

pub fn encode<T: ToBase58>(val: &T) -> String {
    val.to_base58()
}
