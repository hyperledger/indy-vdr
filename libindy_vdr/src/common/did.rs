use regex::Regex;

use crate::common::error::prelude::*;
use crate::utils::base58::FromBase58;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DidMethod(pub String);

lazy_static! {
    pub static ref DEFAULT_LIBINDY_DID: DidValue = DidValue::new("LibindyDid111111111111", None);
}

impl Validatable for DidMethod {
    fn validate(&self) -> VdrResult<()> {
        lazy_static! {
            static ref REGEX_METHOD_NAME: Regex = Regex::new("^[a-z0-9]+$").unwrap();
        }
        if !REGEX_METHOD_NAME.is_match(&self.0) {
            return Err(input_err(format!(
                "Invalid default name: {}. It does not match the DID method name format.",
                self.0
            )));
        }
        Ok(())
    }
}

qualifiable_type!(DidValue);

impl DidValue {
    pub const PREFIX: &'static str = "did";

    pub fn new(did: &str, method: Option<&str>) -> DidValue {
        match method {
            Some(method_) => DidValue(did.to_string()).set_method(&method_),
            None => DidValue(did.to_string()),
        }
    }

    pub fn to_short(&self) -> ShortDidValue {
        ShortDidValue(self.to_unqualified().0)
    }

    pub fn qualify(&self, method: &str) -> DidValue {
        self.set_method(&method)
    }

    pub fn to_unqualified(&self) -> DidValue {
        DidValue(qualifier::to_unqualified(&self.0))
    }

    pub fn is_abbreviatable(&self) -> bool {
        match self.get_method() {
            Some(ref method) if method.starts_with("sov") => true,
            Some(_) => false,
            None => true,
        }
    }

    pub fn from_str(did: &str) -> VdrResult<Self> {
        let did = Self(did.to_owned());
        did.validate()?;
        Ok(did)
    }
}

impl Validatable for DidValue {
    fn validate(&self) -> VdrResult<()> {
        if self.is_fully_qualified() {
            // pass
        } else {
            let did = self.0.from_base58()?;

            if did.len() != 16 && did.len() != 32 {
                return Err(input_err(format!(
                    "Trying to use DID with unexpected length: {}. \
                    The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()
                )));
            }
        }
        Ok(())
    }
}

qualifiable_type!(ShortDidValue);

impl ShortDidValue {
    pub const PREFIX: &'static str = "did";

    pub fn qualify(&self, method: Option<String>) -> DidValue {
        match method {
            Some(method_) => DidValue(self.set_method(&method_).0),
            None => DidValue(self.0.to_string()),
        }
    }
}

impl Validatable for ShortDidValue {
    fn validate(&self) -> VdrResult<()> {
        let did = self.0.from_base58()?;

        if did.len() != 16 && did.len() != 32 {
            return Err(input_err(format!(
                "Trying to use DID with unexpected length: {}. \
                The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()
            )));
        }
        Ok(())
    }
}
