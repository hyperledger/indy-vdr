use super::error::prelude::*;
use crate::utils::base58::{FromBase58, ToBase58};
use crate::utils::crypto::DEFAULT_CRYPTO_TYPE;

pub const VERKEY_ENC_BASE58: &str = "base58";
pub const DEFAULT_VERKEY_ENC: &str = VERKEY_ENC_BASE58;

#[derive(Clone, Debug, PartialEq)]
pub struct VerKey {
    pub key: String,
    pub alg: String,
    pub enc: String,
}

impl VerKey {
    pub fn new(key: &str, alg: Option<&str>, enc: Option<&str>) -> VerKey {
        let alg = match alg {
            Some("") | None => DEFAULT_CRYPTO_TYPE,
            Some(alg) => alg,
        };
        let enc = match enc {
            Some("") | None => DEFAULT_VERKEY_ENC,
            Some(enc) => enc,
        };
        VerKey {
            key: key.to_owned(),
            alg: alg.to_owned(),
            enc: enc.to_owned(),
        }
    }

    pub fn from_str(key: &str) -> LedgerResult<VerKey> {
        Self::from_str_qualified(key, None, None, None)
    }

    pub fn from_str_qualified(
        key: &str,
        dest: Option<&str>,
        alg: Option<&str>,
        enc: Option<&str>,
    ) -> LedgerResult<VerKey> {
        let (key, alg) = if key.contains(':') {
            let splits: Vec<&str> = key.split(':').collect();
            let alg = match splits[1] {
                "" => alg,
                _ => Some(splits[1]),
            };
            (splits[0], alg)
        } else {
            (key, alg)
        };

        if key.starts_with('~') {
            let dest = unwrap_opt_or_return!(
                dest,
                Err(input_err("Destination required for short verkey"))
            );
            let mut result = dest.from_base58()?;
            let mut end = key[1..].from_base58()?;
            result.append(&mut end);
            Ok(VerKey::new(result.to_base58().as_str(), alg, enc))
        } else {
            Ok(VerKey::new(key, alg, enc))
        }
    }

    pub fn long_form(&self) -> String {
        let mut result = self.key.clone();
        result.push(':');
        result.push_str(&self.alg);
        result
    }

    pub fn key_bytes(&self) -> LedgerResult<Vec<u8>> {
        match self.enc.as_str() {
            VERKEY_ENC_BASE58 => self.key.from_base58(),
            _ => Err(input_err("Unsupported verkey format")),
        }
    }
}

impl Into<String> for VerKey {
    fn into(self) -> String {
        if self.alg == DEFAULT_CRYPTO_TYPE {
            self.key
        } else {
            self.long_form()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_empty() {
        assert_eq!(
            VerKey::from_str("").unwrap(),
            VerKey::new("", Some(DEFAULT_CRYPTO_TYPE), Some(DEFAULT_VERKEY_ENC))
        )
    }

    #[test]
    fn from_str_single_colon() {
        assert_eq!(
            VerKey::from_str(":").unwrap(),
            VerKey::new("", Some(DEFAULT_CRYPTO_TYPE), Some(DEFAULT_VERKEY_ENC))
        )
    }

    #[test]
    fn from_str_ends_with_colon() {
        assert_eq!(
            VerKey::from_str("foo:").unwrap(),
            VerKey::new("foo", Some(DEFAULT_CRYPTO_TYPE), Some(DEFAULT_VERKEY_ENC))
        )
    }

    #[test]
    fn from_key_starts_with_colon() {
        assert_eq!(
            VerKey::from_str(":bar").unwrap(),
            VerKey::new("", Some("bar"), Some(DEFAULT_VERKEY_ENC))
        )
    }

    #[test]
    fn from_key_works() {
        assert_eq!(
            VerKey::from_str("foo:bar:baz").unwrap(),
            VerKey::new("foo", Some("bar:bar"), Some(DEFAULT_VERKEY_ENC))
        )
    }

    #[test]
    fn round_trip() {
        assert_eq!(
            VerKey::from_str("foo:bar:baz").unwrap().long_form(),
            "foo:bar:baz"
        )
    }
}
