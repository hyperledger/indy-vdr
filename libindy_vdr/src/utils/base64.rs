use base64::Engine;

use super::ConversionError;

pub fn decode<T: AsRef<[u8]>>(val: T) -> Result<Vec<u8>, ConversionError> {
    Ok(base64::engine::general_purpose::STANDARD
        .decode(val)
        .map_err(|err| ("Error decoding base64 data", err))?)
}

pub fn encode<T: AsRef<[u8]>>(val: T) -> String {
    base64::engine::general_purpose::STANDARD.encode(val)
}

pub fn decode_urlsafe<T: AsRef<[u8]>>(val: T) -> Result<Vec<u8>, ConversionError> {
    Ok(base64::engine::general_purpose::URL_SAFE
        .decode(val)
        .map_err(|err| ("Error decoding base64-URL data", err))?)
}

pub fn encode_urlsafe<T: AsRef<[u8]>>(val: T) -> String {
    base64::engine::general_purpose::URL_SAFE.encode(val)
}
