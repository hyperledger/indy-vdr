use super::constants::{GET_SCHEMA, SCHEMA};
use super::{get_sp_key_marker, ProtocolVersion, RequestType};
use crate::common::did::{DidValue, ShortDidValue};
use crate::common::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::validation::Validatable;

use std::collections::HashSet;

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

qualifiable_type!(SchemaId);

impl SchemaId {
    pub const DELIMITER: &'static str = ":";
    pub const PREFIX: &'static str = "schema";
    pub const MARKER: &'static str = "2";

    pub fn new(did: &DidValue, name: &str, version: &str) -> Self {
        let id = Self(format!(
            "{}{}{}{}{}{}{}",
            did.0,
            Self::DELIMITER,
            Self::MARKER,
            Self::DELIMITER,
            name,
            Self::DELIMITER,
            version
        ));
        match did.get_method() {
            Some(method) => id.set_method(&method),
            None => id,
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, String)> {
        let parts = self
            .0
            .split_terminator(Self::DELIMITER)
            .collect::<Vec<&str>>();

        if parts.len() == 1 {
            // 1
            return None;
        }

        if parts.len() == 4 {
            // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let name = parts[2].to_string();
            let version = parts[3].to_string();
            return Some((DidValue(did), name, version));
        }

        if parts.len() == 8 {
            // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[2..5].join(Self::DELIMITER);
            let name = parts[6].to_string();
            let version = parts[7].to_string();
            return Some((DidValue(did), name, version));
        }

        None
    }

    pub fn qualify(&self, method: &str) -> SchemaId {
        match self.parts() {
            Some((did, name, version)) => SchemaId::new(&did.qualify(method), &name, &version),
            None => self.clone(),
        }
    }

    pub fn to_unqualified(&self) -> SchemaId {
        match self.parts() {
            Some((did, name, version)) => SchemaId::new(&did.to_unqualified(), &name, &version),
            None => self.clone(),
        }
    }

    pub fn from_str(schema_id: &str) -> VdrResult<Self> {
        let schema_id = Self(schema_id.to_owned());
        schema_id.validate()?;
        Ok(schema_id)
    }
}

impl Validatable for SchemaId {
    fn validate(&self) -> VdrResult<()> {
        if self.0.parse::<i32>().is_ok() {
            return Ok(());
        }

        self.parts().ok_or_else(|| {
            input_err(format!(
                "SchemaId validation failed: {:?}, doesn't match pattern",
                self.0
            ))
        })?;

        Ok(())
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
    fn validate(&self) -> VdrResult<()> {
        self.attr_names.validate()?;
        self.id.validate()?;
        if let Some((_, name, version)) = self.id.parts() {
            if name != self.name {
                return Err(input_err(format!(
                    "Inconsistent Schema Id and Schema Name: {:?} and {}",
                    self.id, self.name,
                )));
            }
            if version != self.version {
                return Err(input_err(format!(
                    "Inconsistent Schema Id and Schema Version: {:?} and {}",
                    self.id, self.version,
                )));
            }
        }
        Ok(())
    }
}

impl Validatable for AttributeNames {
    fn validate(&self) -> VdrResult<()> {
        if self.0.is_empty() {
            return Err(input_err("Empty list of Schema attributes has been passed"));
        }

        if self.0.len() > MAX_ATTRIBUTES_COUNT {
            return Err(input_err(format!(
                "The number of Schema attributes {} cannot be greater than {}",
                self.0.len(),
                MAX_ATTRIBUTES_COUNT
            )));
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
