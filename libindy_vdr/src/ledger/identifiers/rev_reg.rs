use regex::Regex;

use super::cred_def::CredentialDefinitionId;
use crate::common::did::DidValue;
use crate::common::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

pub const CL_ACCUM: &str = "CL_ACCUM";

lazy_static! {
    static ref QUALIFIED_REV_REG_ID: Regex = Regex::new("(^revreg:(?P<method>[a-z0-9]+):)?(?P<did>.+):4:(?P<cred_def_id>.+):(?P<rev_reg_type>.+):(?P<tag>.+)$").unwrap();
}

qualifiable_type!(RevocationRegistryId);

impl RevocationRegistryId {
    pub const PREFIX: &'static str = "revreg";
    pub const DELIMITER: &'static str = ":";
    pub const MARKER: &'static str = "4";

    pub fn new(
        did: &DidValue,
        cred_def_id: &CredentialDefinitionId,
        rev_reg_type: &str,
        tag: &str,
    ) -> RevocationRegistryId {
        let id = RevocationRegistryId(format!(
            "{}{}{}{}{}{}{}{}{}",
            did.0,
            Self::DELIMITER,
            Self::MARKER,
            Self::DELIMITER,
            cred_def_id.0,
            Self::DELIMITER,
            rev_reg_type,
            Self::DELIMITER,
            tag
        ));
        match did.get_method() {
            Some(method) => RevocationRegistryId(qualifier::qualify(&id.0, Self::PREFIX, &method)),
            None => id,
        }
    }

    pub fn parts(&self) -> Option<(DidValue, CredentialDefinitionId, String, String)> {
        match QUALIFIED_REV_REG_ID.captures(&self.0) {
            Some(caps) => Some((
                DidValue(caps["did"].to_string()),
                CredentialDefinitionId(caps["cred_def_id"].to_string()),
                caps["rev_reg_type"].to_string(),
                caps["tag"].to_string(),
            )),
            None => None,
        }
    }

    pub fn to_unqualified(&self) -> RevocationRegistryId {
        match self.parts() {
            Some((did, cred_def_id, rev_reg_type, tag)) => RevocationRegistryId::new(
                &did.to_unqualified(),
                &cred_def_id.to_unqualified(),
                &rev_reg_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn from_str(rev_reg_def_id: &str) -> VdrResult<Self> {
        let rev_reg_def_id = Self(rev_reg_def_id.to_owned());
        rev_reg_def_id.validate()?;
        Ok(rev_reg_def_id)
    }
}

impl Validatable for RevocationRegistryId {
    fn validate(&self) -> VdrResult<()> {
        self.parts().ok_or(input_err(format!(
            "Revocation Registry Id validation failed: {:?}, doesn't match pattern",
            self.0
        )))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _rev_reg_type() -> String {
        "CL_ACCUM".to_string()
    }

    fn _tag() -> String {
        "TAG_1".to_string()
    }

    fn _did_qualified() -> DidValue {
        DidValue("did:sov:NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _cred_def_id_unqualified() -> CredentialDefinitionId {
        CredentialDefinitionId(
            "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string(),
        )
    }

    fn _cred_def_id_qualified() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    fn _rev_reg_id_unqualified() -> RevocationRegistryId {
        RevocationRegistryId("NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string())
    }

    fn _rev_reg_id_qualified() -> RevocationRegistryId {
        RevocationRegistryId("revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string())
    }

    mod to_unqualified {
        use super::*;

        #[test]
        fn test_rev_reg_id_parts_for_id_as_unqualified() {
            assert_eq!(
                _rev_reg_id_unqualified(),
                _rev_reg_id_unqualified().to_unqualified()
            );
        }

        #[test]
        fn test_rev_reg_id_parts_for_id_as_qualified() {
            assert_eq!(
                _rev_reg_id_unqualified(),
                _rev_reg_id_qualified().to_unqualified()
            );
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_rev_reg_id_parts_for_id_as_unqualified() {
            let (did, cred_def_id, rev_reg_type, tag) = _rev_reg_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_cred_def_id_unqualified(), cred_def_id);
            assert_eq!(_rev_reg_type(), rev_reg_type);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_rev_reg_id_parts_for_id_as_qualified() {
            let (did, cred_def_id, rev_reg_type, tag) = _rev_reg_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_cred_def_id_qualified(), cred_def_id);
            assert_eq!(_rev_reg_type(), rev_reg_type);
            assert_eq!(_tag(), tag);
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn test_validate_rev_reg_id_as_unqualified() {
            _rev_reg_id_unqualified().validate().unwrap();
        }

        #[test]
        fn test_validate_rev_reg_id_as_fully_qualified() {
            _rev_reg_id_qualified().validate().unwrap();
        }
    }
}
