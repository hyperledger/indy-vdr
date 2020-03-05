use regex::Regex;

use crate::utils::base58::FromBase58;
use crate::utils::qualifier::Qualifiable;
use crate::utils::validation::{Validatable, ValidationError};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DidMethod(pub String);

lazy_static! {
    pub static ref DEFAULT_LIBINDY_DID: DidValue = DidValue::new("LibindyDid111111111111", None);
}

impl Validatable for DidMethod {
    fn validate(&self) -> Result<(), ValidationError> {
        lazy_static! {
            static ref REGEX_METHOD_NAME: Regex = Regex::new("^[a-z0-9]+$").unwrap();
        }
        if !REGEX_METHOD_NAME.is_match(&self.0) {
            return Err(invalid!(
                "Invalid default name: {}. It does not match the DID method name format.",
                self.0
            ));
        }
        Ok(())
    }
}

qualifiable_type!(DidValue);

impl Qualifiable for DidValue {
    fn prefix() -> &'static str {
        "did"
    }
}

impl DidValue {
    pub fn new(did: &str, method: Option<&str>) -> DidValue {
        DidValue::combine(method, did)
    }

    pub fn to_short(&self) -> ShortDidValue {
        ShortDidValue(self.to_unqualified().0)
    }

    pub fn is_abbreviatable(&self) -> bool {
        match self.get_method() {
            Some(ref method) if method.starts_with("sov") => true,
            Some(_) => false,
            None => true,
        }
    }
}

impl Validatable for DidValue {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.is_fully_qualified() {
            // pass
        } else {
            let did = self.from_base58()?;
            if did.len() != 16 && did.len() != 32 {
                return Err(invalid!(
                    "Trying to use DID with unexpected length: {}. \
                    The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()
                ));
            }
        }
        Ok(())
    }
}

qualifiable_type!(ShortDidValue);

impl Qualifiable for ShortDidValue {
    fn prefix() -> &'static str {
        "did"
    }
}

impl ShortDidValue {
    pub fn qualify(&self, method: Option<String>) -> DidValue {
        DidValue::combine(method.as_ref().map(String::as_str), self.as_str())
    }
}

impl Validatable for ShortDidValue {
    fn validate(&self) -> Result<(), ValidationError> {
        let did = self.from_base58()?;
        if did.len() != 16 && did.len() != 32 {
            return Err(invalid!(
                "Trying to use DID with unexpected length: {}. \
                The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()
            ));
        }
        Ok(())
    }
}
