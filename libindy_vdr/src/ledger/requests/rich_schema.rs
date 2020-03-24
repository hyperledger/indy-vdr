use super::RequestType;
use crate::ledger::constants::{
    GET_RICH_SCHEMA_BY_ID, GET_RICH_SCHEMA_BY_METADATA, RS_POSSIBLE_TYPES,
    RICH_SCHEMA_ENCODING, RICH_SCHEMA_MAPPING, RICH_SCHEMA_CTX,
    RICH_SCHEMA_CRED_DEF, RICH_SCHEMA_PRES_DEF, RICH_SCHEMA
};
use crate::ledger::identifiers::rich_schema::RichSchemaId;
use crate::utils::validation::{Validatable, ValidationError};

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RSContent(pub String);

impl Validatable for RSContent {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: Add JSON-LD validation if needed
        return Ok(());
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RichSchema {
    pub id: RichSchemaId,
    pub content: RSContent,
    pub rs_name: String,
    pub rs_version: String,
    pub rs_type: String,
    pub ver: String,
}

impl RichSchema {
    pub fn new(
        id: RichSchemaId,
        content: RSContent,
        rs_name: String,
        rs_version: String,
        rs_type: String,
        ver: String,
    ) -> Self {
        Self {
            id,
            content,
            rs_name,
            rs_version,
            rs_type,
            ver,
        }
    }
}

impl Validatable for RichSchema {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: add specific validation
        if ! RS_POSSIBLE_TYPES.contains(&self.rs_type.as_str()) {
            return Err(ValidationError::from(format!("Should be one of {:?}", RS_POSSIBLE_TYPES)));
        }
        return self.id.validate();
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RichSchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RichSchemaId,
    pub content: RSContent,
    pub rs_name: String,
    pub rs_version: String,
    pub rs_type: String,
    pub ver: String,
}

impl RichSchemaOperation {
    pub fn new(rs_schema: RichSchema) -> RichSchemaOperation {
        RichSchemaOperation {
            _type:Self::get_txn_type().to_string(),
            id: rs_schema.id,
            content: rs_schema.content,
            rs_name: rs_schema.rs_name,
            rs_version: rs_schema.rs_version,
            rs_type: rs_schema.rs_type,
            ver: rs_schema.ver,
        }
    }
}

impl RequestType for RichSchemaOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RSEncodingOperation(pub RichSchemaOperation);

impl RequestType for RSEncodingOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_ENCODING
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RSMappingOperation(pub RichSchemaOperation);

impl RequestType for RSMappingOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_MAPPING
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RSContextOperation(pub RichSchemaOperation);

impl RequestType for RSContextOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_CTX
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RSCredDefOperation(pub RichSchemaOperation);

impl RequestType for RSCredDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_CRED_DEF
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RSPresDefOperation(pub RichSchemaOperation);

impl RequestType for RSPresDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_PRES_DEF
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum RSType {
    #[serde(rename="sch")]
    Sch,
    #[serde(rename="map")]
    Map,
    #[serde(rename="ctx")]
    Ctx,
    #[serde(rename="enc")]
    Enc,
    #[serde(rename="cdf")]
    Cdf,
    #[serde(rename="pdf")]
    Pdf,
}

// Get RichSchema object from ledger using RichSchema's ID.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetRichSchemaById {
    #[serde(rename = "type")]
    pub id: RichSchemaId,
}

impl GetRichSchemaById {
    pub fn new(id: RichSchemaId) -> Self {
        Self { id }
    }
}

impl Validatable for GetRichSchemaById {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: add specific validation if needed
        return self.id.validate();
    }
}

#[derive(Serialize, Debug)]
pub struct GetRichSchemaByIdOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RichSchemaId,
}

impl GetRichSchemaByIdOperation {
    pub fn new(get_rs: GetRichSchemaById) -> Self {
        Self {
            _type: Self::get_txn_type().to_string(),
            id: get_rs.id,
        }
    }
}

impl Validatable for GetRichSchemaByIdOperation {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: add specific validation
        return self.id.validate();
    }
}

impl RequestType for GetRichSchemaByIdOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_RICH_SCHEMA_BY_ID
    }
}

// Get RichSchema object from ledger using metadata:
//      rs_type: Rich Schema object's type enum
//      rs_name: Rich Schema object's name,
//      rs_version: Rich Schema object's version,

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetRichSchemaByMetadata {
    pub rs_type: String,
    pub rs_name: String,
    pub rs_version: String,
}

impl GetRichSchemaByMetadata {
    pub fn new(rs_type: String, rs_name: String, rs_version: String) -> Self {
        Self {
            rs_type,
            rs_name,
            rs_version,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GetRichSchemaByMetadataOperation {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "rsType")]
    pub rs_type: String,
    #[serde(rename = "rsName")]
    pub rs_name: String,
    #[serde(rename = "rsVersion")]
    pub rs_version: String,
}

impl GetRichSchemaByMetadataOperation {
    pub fn new(get_rs: GetRichSchemaByMetadata) -> Self {
        Self {
            _type: Self::get_txn_type().to_string(),
            rs_type: get_rs.rs_type,
            rs_name: get_rs.rs_name,
            rs_version: get_rs.rs_version,
        }
    }
}

impl RequestType for GetRichSchemaByMetadataOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_RICH_SCHEMA_BY_METADATA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::constants::RS_SCHEMA_TYPE_VALUE;

    fn _rich_schema_id() -> RichSchemaId {
        RichSchemaId::new("did:sov:some_hash_value".to_string())
    }

    fn _get_rs_by_id() -> GetRichSchemaById {
        GetRichSchemaById::new(_rich_schema_id())
    }

    fn _get_rs_by_metadata() -> GetRichSchemaByMetadata {
        GetRichSchemaByMetadata::new(
            RS_SCHEMA_TYPE_VALUE.to_string(),
            "test_rich_schema".to_string(),
            "first_version".to_string(),
        )
    }

    fn _get_rs_by_metadata_op() -> GetRichSchemaByMetadataOperation {
        GetRichSchemaByMetadataOperation::new(_get_rs_by_metadata())
    }

    fn _rs_schema() -> RichSchema {
        RichSchema::new(
            _rich_schema_id(),
            RSContent(r#"{"json": "ld"; "valid": "object"}"#.to_string()),
            "test_rich_schema".to_string(),
            "first_version".to_string(),
            RS_SCHEMA_TYPE_VALUE.to_string(),
            "1".to_string(),
        )
    }

    fn _get_rs_op_by_id() -> GetRichSchemaByIdOperation {
        GetRichSchemaByIdOperation::new(_get_rs_by_id())
    }

    #[test]
    fn test_check_type_get_rs_by_id_op() {
        assert_eq!(_get_rs_op_by_id()._type, GET_RICH_SCHEMA_BY_ID)
    }
    #[test]
    fn test_check_type_get_rs_by_metadata_op() {
        assert_eq!(_get_rs_by_metadata_op()._type, GET_RICH_SCHEMA_BY_METADATA)
    }

    #[test]
    fn test_fail_on_wrong_rs_type() {
        let mut rs_schema = _rs_schema();
        rs_schema.rs_type = "SomeOtherType".to_string();
        let err = rs_schema.validate().unwrap_err();
        assert!(err.to_string().contains("Should be one of"));
    }
}
