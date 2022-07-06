pub use indy_data_types::anoncreds::schema::{
    AttributeNames, Schema, SchemaV1, MAX_ATTRIBUTES_COUNT,
};

use super::constants::{GET_SCHEMA, SCHEMA};
use super::did::ShortDidValue;
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::error::prelude::*;

use std::collections::HashSet;

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
                &*self.dest, marker, self.data.name, self.data.version
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
