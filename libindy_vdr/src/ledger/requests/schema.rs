use super::constants::{GET_SCHEMA, SCHEMA};
use super::identifiers::schema::SchemaId;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::did::ShortDidValue;
use crate::common::error::prelude::*;
use crate::utils::validation::{Validatable, ValidationError};

use std::collections::HashSet;

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ver")]
pub enum Schema {
    #[serde(rename = "1.0")]
    SchemaV1(SchemaV1),
}

impl Schema {
    pub fn to_unqualified(self) -> Schema {
        match self {
            Schema::SchemaV1(schema) => {
                Schema::SchemaV1(SchemaV1 {
                    id: schema.id.to_unqualified(),
                    name: schema.name,
                    version: schema.version,
                    attr_names: schema.attr_names,
                    seq_no: schema.seq_no,
                })
            }
        }
    }
}

impl Validatable for Schema {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Schema::SchemaV1(schema) => schema.validate(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaV1 {
    pub id: SchemaId,
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: AttributeNames,
    pub seq_no: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeNames(pub HashSet<String>);

impl AttributeNames {
    pub fn new() -> Self {
        AttributeNames(HashSet::new())
    }
}

impl From<HashSet<String>> for AttributeNames {
    fn from(attrs: HashSet<String>) -> Self {
        AttributeNames(attrs)
    }
}

impl Into<HashSet<String>> for AttributeNames {
    fn into(self) -> HashSet<String> {
        self.0
    }
}

impl Validatable for SchemaV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        self.attr_names.validate()?;
        self.id.validate()?;
        if let Some((_, _, name, version)) = self.id.parts() {
            if name != self.name {
                return Err(invalid!(
                    "Inconsistent Schema Id and Schema Name: {:?} and {}",
                    self.id,
                    self.name,
                ));
            }
            if version != self.version {
                return Err(invalid!(
                    "Inconsistent Schema Id and Schema Version: {:?} and {}",
                    self.id,
                    self.version,
                ));
            }
        }
        Ok(())
    }
}

impl Validatable for AttributeNames {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.0.is_empty() {
            return Err(invalid!("Empty list of Schema attributes has been passed"));
        }

        if self.0.len() > MAX_ATTRIBUTES_COUNT {
            return Err(invalid!(
                "The number of Schema attributes {} cannot be greater than {}",
                self.0.len(),
                MAX_ATTRIBUTES_COUNT
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct SchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: SchemaOperationData,
}

impl SchemaOperation {
    pub fn new(data: SchemaOperationData) -> Self {
        Self {
            data,
            _type: Self::get_txn_type().to_string(),
        }
    }
}

impl RequestType for SchemaOperation {
    fn get_txn_type<'a>() -> &'a str {
        SCHEMA
    }
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub struct SchemaOperationData {
    pub name: String,
    pub version: String,
    pub attr_names: HashSet<String>,
}

impl SchemaOperationData {
    pub fn new(name: String, version: String, attr_names: HashSet<String>) -> Self {
        Self {
            name,
            version,
            attr_names,
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetSchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    pub data: GetSchemaOperationData,
}

impl GetSchemaOperation {
    pub fn new(dest: ShortDidValue, data: GetSchemaOperationData) -> Self {
        Self {
            _type: Self::get_txn_type().to_string(),
            dest,
            data,
        }
    }
}

impl RequestType for GetSchemaOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_SCHEMA
    }

    fn get_sp_key(&self, protocol_version: ProtocolVersion) -> VdrResult<Option<Vec<u8>>> {
        let marker = get_sp_key_marker(2, protocol_version);
        Ok(Some(
            format!(
                "{}:{}:{}:{}",
                self.dest.to_string(),
                marker,
                self.data.name,
                self.data.version
            )
            .as_bytes()
            .to_vec(),
        ))
    }
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub struct GetSchemaOperationData {
    pub name: String,
    pub version: String,
}

impl GetSchemaOperationData {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

/*
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetSchemaReplyResult {
    GetSchemaReplyResultV0(GetSchemaResultV0),
    GetSchemaReplyResultV1(GetReplyResultV1<GetSchemaResultDataV1>),
}

impl ReplyType for GetSchemaReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_SCHEMA
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultV0 {
    pub seq_no: u32,
    pub data: SchemaOperationData,
    pub dest: ShortDidValue,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataV1 {
    pub ver: String,
    pub id: SchemaId,
    pub schema_name: String,
    pub schema_version: String,
    pub value: GetSchemaResultDataValueV1,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataValueV1 {
    pub attr_names: HashSet<String>,
}
*/

#[cfg(test)]
mod test_schema_validation {
    use super::*;

    fn _schema_id_qualified() -> SchemaId {
        SchemaId("schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    #[test]
    fn test_valid_schema() {
        let schema_json = json!({
            "id": _schema_id_qualified(),
            "name": "gvt",
            "ver": "1.0",
            "version": "1.0",
            "attrNames": ["aaa", "bbb", "ccc"],
        })
        .to_string();

        let schema: SchemaV1 = serde_json::from_str(&schema_json).unwrap();
        schema.validate().unwrap();
        assert_eq!(schema.name, "gvt");
        assert_eq!(schema.version, "1.0");
    }

    #[test]
    fn test_invalid_name_schema() {
        let schema_json = json!({
            "id": _schema_id_qualified(),
            "name": "gvt1",
            "ver": "1.0",
            "version": "1.0",
            "attrNames": ["aaa", "bbb", "ccc"],
        })
        .to_string();

        let schema: SchemaV1 = serde_json::from_str(&schema_json).unwrap();
        schema.validate().unwrap_err();
    }

    #[test]
    fn test_invalid_version_schema() {
        let schema_json = json!({
            "id": _schema_id_qualified(),
            "name": "gvt",
            "ver": "1.0",
            "version": "1.1",
            "attrNames": ["aaa", "bbb", "ccc"],
        })
        .to_string();

        let schema: SchemaV1 = serde_json::from_str(&schema_json).unwrap();
        schema.validate().unwrap_err();
    }
}
