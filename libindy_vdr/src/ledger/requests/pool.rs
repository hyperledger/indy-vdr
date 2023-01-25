use super::constants::{POOL_CONFIG, POOL_RESTART, POOL_UPGRADE};
use super::RequestType;

use std::collections::HashMap;

#[derive(Serialize, PartialEq, Debug)]
pub struct PoolConfigOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub writes: bool,
    pub force: bool,
}

impl PoolConfigOperation {
    pub fn new(writes: bool, force: bool) -> PoolConfigOperation {
        PoolConfigOperation {
            _type: Self::get_txn_type().to_string(),
            writes,
            force,
        }
    }
}

impl RequestType for PoolConfigOperation {
    fn get_txn_type<'a>() -> &'a str {
        POOL_CONFIG
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct PoolRestartOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub action: String,
    //start, cancel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
}

impl PoolRestartOperation {
    pub fn new(action: &str, datetime: Option<String>) -> PoolRestartOperation {
        PoolRestartOperation {
            _type: Self::get_txn_type().to_string(),
            action: action.to_string(),
            datetime,
        }
    }
}

impl RequestType for PoolRestartOperation {
    fn get_txn_type<'a>() -> &'a str {
        POOL_RESTART
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct PoolUpgradeOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub name: String,
    pub version: String,
    pub action: String,
    //start, cancel
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justification: Option<String>,
    pub reinstall: bool,
    pub force: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

impl PoolUpgradeOperation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        version: &str,
        action: &str,
        sha256: &str,
        timeout: Option<u32>,
        schedule: Option<HashMap<String, String>>,
        justification: Option<&str>,
        reinstall: bool,
        force: bool,
        package: Option<&str>,
    ) -> PoolUpgradeOperation {
        PoolUpgradeOperation {
            _type: Self::get_txn_type().to_string(),
            name: name.to_string(),
            version: version.to_string(),
            action: action.to_string(),
            sha256: sha256.to_string(),
            timeout,
            schedule,
            justification: justification.map(String::from),
            reinstall,
            force,
            package: package.map(String::from),
        }
    }
}

impl RequestType for PoolUpgradeOperation {
    fn get_txn_type<'a>() -> &'a str {
        POOL_UPGRADE
    }
}

pub type Schedule = HashMap<String, String>;
