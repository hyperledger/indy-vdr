use super::did::DidValue;
use super::schema::SchemaId;
use crate::common::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

pub const CL_SIGNATURE_TYPE: &str = "CL";

qualifiable_type!(CredentialDefinitionId);

impl CredentialDefinitionId {
    pub const DELIMITER: &'static str = ":";
    pub const PREFIX: &'static str = "creddef";
    pub const MARKER: &'static str = "3";

    pub fn new(
        did: &DidValue,
        schema_id: &SchemaId,
        signature_type: &str,
        tag: &str,
    ) -> CredentialDefinitionId {
        let tag = if tag.is_empty() {
            format!("")
        } else {
            format!("{}{}", Self::DELIMITER, tag)
        };
        let id = CredentialDefinitionId(format!(
            "{}{}{}{}{}{}{}{}",
            did.0,
            Self::DELIMITER,
            Self::MARKER,
            Self::DELIMITER,
            signature_type,
            Self::DELIMITER,
            schema_id.0,
            tag
        ));
        match did.get_method() {
            Some(method) => id.set_method(&method),
            None => id,
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, SchemaId, String)> {
        let parts = self
            .0
            .split_terminator(Self::DELIMITER)
            .collect::<Vec<&str>>();

        if parts.len() == 4 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 5 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = parts[4].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 7 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(Self::DELIMITER);
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 8 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(Self::DELIMITER);
            let tag = parts[7].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 9 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:3:tag
            let did = parts[2..5].join(Self::DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7].to_string();
            let tag = parts[8].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 16 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[2..5].join(Self::DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7..15].join(Self::DELIMITER);
            let tag = parts[15].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        None
    }

    pub fn issuer_did(&self) -> Option<DidValue> {
        self.parts().map(|(did, _, _, _)| did)
    }

    pub fn qualify(&self, method: &str) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => CredentialDefinitionId::new(
                &did.qualify(method),
                &schema_id.qualify(method),
                &signature_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn to_unqualified(&self) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => CredentialDefinitionId::new(
                &did.to_unqualified(),
                &schema_id.to_unqualified(),
                &signature_type,
                &tag,
            ),
            None => self.clone(),
        }
    }

    pub fn from_str(cred_def_id: &str) -> VdrResult<Self> {
        let cred_def_id = Self(cred_def_id.to_owned());
        cred_def_id.validate()?;
        Ok(cred_def_id)
    }
}

impl Validatable for CredentialDefinitionId {
    fn validate(&self) -> VdrResult<()> {
        self.parts().ok_or(input_err(format!(
            "Credential Definition Id validation failed: {:?}, doesn't match pattern",
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

    fn _signature_type() -> String {
        "CL".to_string()
    }

    fn _tag() -> String {
        "tag".to_string()
    }

    fn _did_qualified() -> DidValue {
        DidValue("did:sov:NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _schema_id_seq_no() -> SchemaId {
        SchemaId("1".to_string())
    }

    fn _schema_id_unqualified() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _schema_id_qualified() -> SchemaId {
        SchemaId("schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _cred_def_id_unqualified() -> CredentialDefinitionId {
        CredentialDefinitionId(
            "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string(),
        )
    }

    fn _cred_def_id_unqualified_with_schema_as_seq_no() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _cred_def_id_unqualified_with_schema_as_seq_no_without_tag() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1".to_string())
    }

    fn _cred_def_id_unqualified_without_tag() -> CredentialDefinitionId {
        CredentialDefinitionId(
            "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string(),
        )
    }

    fn _cred_def_id_qualified_with_schema_as_seq_no() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _cred_def_id_qualified() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    mod to_unqualified {
        use super::*;

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified() {
            assert_eq!(
                _cred_def_id_unqualified(),
                _cred_def_id_unqualified().to_unqualified()
            );
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag() {
            assert_eq!(
                _cred_def_id_unqualified_without_tag(),
                _cred_def_id_unqualified_without_tag().to_unqualified()
            );
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag_with_schema_as_seq_no() {
            assert_eq!(
                _cred_def_id_unqualified_with_schema_as_seq_no(),
                _cred_def_id_unqualified_with_schema_as_seq_no().to_unqualified()
            );
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag_with_schema_as_seq_no_without_tag(
        ) {
            assert_eq!(
                _cred_def_id_unqualified_with_schema_as_seq_no_without_tag(),
                _cred_def_id_unqualified_with_schema_as_seq_no_without_tag().to_unqualified()
            );
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified() {
            assert_eq!(
                _cred_def_id_unqualified(),
                _cred_def_id_qualified().to_unqualified()
            );
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified_with_schema_as_seq_no() {
            assert_eq!(
                _cred_def_id_unqualified_with_schema_as_seq_no(),
                _cred_def_id_qualified_with_schema_as_seq_no().to_unqualified()
            );
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag() {
            let (did, signature_type, schema_id, tag) =
                _cred_def_id_unqualified_without_tag().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq() {
            let (did, signature_type, schema_id, tag) =
                _cred_def_id_unqualified_with_schema_as_seq_no()
                    .parts()
                    .unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq_without_tag() {
            let (did, signature_type, schema_id, tag) =
                _cred_def_id_unqualified_with_schema_as_seq_no_without_tag()
                    .parts()
                    .unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_qualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified_with_schema_as_seq() {
            let (did, signature_type, schema_id, tag) =
                _cred_def_id_qualified_with_schema_as_seq_no()
                    .parts()
                    .unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(_tag(), tag);
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn test_validate_cred_def_id_as_unqualified() {
            _cred_def_id_unqualified().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_without_tag() {
            _cred_def_id_unqualified_without_tag().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_with_schema_as_seq_no() {
            _cred_def_id_unqualified_with_schema_as_seq_no()
                .validate()
                .unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_with_schema_as_seq_no_without_tag() {
            _cred_def_id_unqualified_with_schema_as_seq_no_without_tag()
                .validate()
                .unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_fully_qualified() {
            _cred_def_id_qualified().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_fully_qualified_with_schema_as_seq_no() {
            _cred_def_id_qualified_with_schema_as_seq_no()
                .validate()
                .unwrap();
        }
    }
}
