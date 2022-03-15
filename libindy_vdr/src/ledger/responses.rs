use crate::utils::did::DidValue;
use serde::Deserialize;
use std::collections::HashMap;

pub enum ResponseTypes {
    GetNymResult(GetNymResult),
    GetSchemaResult(GetSchemaResult),
    GetClaimDefResult(GetClaimDefResult),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GetNymResult {
    GetNymResultV0(GetNymResultV0),
    GetNymResultV1(GetNymResultV1),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct GetNymResultV0 {
    pub identifier: Option<DidValue>,
    pub dest: DidValue,
    pub role: Option<String>,
    pub verkey: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNymResultV1 {
    pub identifier: Option<DidValue>,
    pub dest: DidValue,
    pub role: Option<String>,
    pub verkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diddoc_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetSchemaResult {
    pub attr_names: Vec<String>,
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetClaimDefResult {
    pub ref_schema_attributes: Vec<String>,
    pub ref_schema_from: String,
    pub ref_schema_id: String,
    pub ref_schema_name: String,
    pub ref_schema_txn_seq_no: u32,
    pub ref_schema_txn_time: String,
    pub ref_schema_version: String,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Endpoint {
    pub endpoint: HashMap<String, String>,
}
