pub use indy_data_types::anoncreds::rich_schema::{RSContent, RSType, RichSchema};

use super::RequestType;
use crate::ledger::constants::{
    GET_RICH_SCHEMA_BY_ID, GET_RICH_SCHEMA_BY_METADATA, RICH_SCHEMA, RICH_SCHEMA_CRED_DEF,
    RICH_SCHEMA_CTX, RICH_SCHEMA_ENCODING, RICH_SCHEMA_MAPPING, RICH_SCHEMA_PRES_DEF,
};
use crate::ledger::identifiers::RichSchemaId;
use crate::utils::{Validatable, ValidationError};

#[macro_export]
macro_rules! build_rs_operation {
    ($self:ident, $operation:ident, $identifier:expr, $rich_schema:expr) => {{
        $self.build(
            $operation(RichSchemaBaseOperation::new(
                $rich_schema,
                $operation::get_txn_type().to_string(),
            )),
            Some($identifier),
        )
    }};
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RichSchemaBaseOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RichSchemaId,
    pub content: RSContent,
    pub rs_name: String,
    pub rs_version: String,
    pub rs_type: String,
    pub ver: String,
}

impl RichSchemaBaseOperation {
    pub fn new(rs_schema: RichSchema, txn_type: String) -> RichSchemaBaseOperation {
        RichSchemaBaseOperation {
            _type: txn_type,
            id: rs_schema.id,
            content: rs_schema.content,
            rs_name: rs_schema.rs_name,
            rs_version: rs_schema.rs_version,
            rs_type: rs_schema.rs_type,
            ver: rs_schema.ver,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RichSchemaOperation(pub RichSchemaBaseOperation);

impl RequestType for RichSchemaOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA
    }
}

#[derive(Serialize, Debug)]
pub struct RSEncodingOperation(pub RichSchemaBaseOperation);

impl RequestType for RSEncodingOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_ENCODING
    }
}

#[derive(Serialize, Debug)]
pub struct RSMappingOperation(pub RichSchemaBaseOperation);

impl RequestType for RSMappingOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_MAPPING
    }
}

#[derive(Serialize, Debug)]
pub struct RSContextOperation(pub RichSchemaBaseOperation);

impl RequestType for RSContextOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_CTX
    }
}

#[derive(Serialize, Debug)]
pub struct RSCredDefOperation(pub RichSchemaBaseOperation);

impl RequestType for RSCredDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_CRED_DEF
    }
}

#[derive(Serialize, Debug)]
pub struct RSPresDefOperation(pub RichSchemaBaseOperation);

impl RequestType for RSPresDefOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA_PRES_DEF
    }
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
        self.id.validate()
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
        self.id.validate()
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
#[serde(rename_all = "camelCase")]
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
}
