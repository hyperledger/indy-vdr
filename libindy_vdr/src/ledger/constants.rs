use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

use crate::common::error::{input_err, VdrError};

pub const NODE: &str = "0";
pub const NYM: &str = "1";
pub const GET_TXN: &str = "3";
pub const TXN_AUTHR_AGRMT: &str = "4"; // TODO Use nonabbreviated names as in updated design
pub const TXN_AUTHR_AGRMT_AML: &str = "5";
pub const GET_TXN_AUTHR_AGRMT: &str = "6";
pub const GET_TXN_AUTHR_AGRMT_AML: &str = "7";
pub const DISABLE_ALL_TXN_AUTHR_AGRMTS: &str = "8";
pub const LEDGERS_FREEZE: &str = "9";
pub const GET_FROZEN_LEDGERS: &str = "10";
pub const ATTRIB: &str = "100";
pub const SCHEMA: &str = "101";
pub const CRED_DEF: &str = "102";
pub const GET_ATTR: &str = "104";
pub const GET_NYM: &str = "105";
pub const GET_SCHEMA: &str = "107";
pub const GET_CRED_DEF: &str = "108";
pub const POOL_UPGRADE: &str = "109";
pub const POOL_RESTART: &str = "118";
pub const POOL_CONFIG: &str = "111";
pub const REVOC_REG_DEF: &str = "113";
pub const REVOC_REG_ENTRY: &str = "114";
pub const GET_REVOC_REG_DEF: &str = "115";
pub const GET_REVOC_REG: &str = "116";
pub const GET_REVOC_REG_DELTA: &str = "117";
pub const GET_VALIDATOR_INFO: &str = "119";
pub const AUTH_RULE: &str = "120";
pub const GET_AUTH_RULE: &str = "121";
pub const AUTH_RULES: &str = "122";
pub const FLAG: &str = "130";
pub const GET_FLAG: &str = "131";

// RichSchema objects
pub const RICH_SCHEMA_CTX: &str = "200";
pub const RICH_SCHEMA: &str = "201";
pub const RICH_SCHEMA_ENCODING: &str = "202";
pub const RICH_SCHEMA_MAPPING: &str = "203";
pub const RICH_SCHEMA_CRED_DEF: &str = "204";
pub const RICH_SCHEMA_PRES_DEF: &str = "205";

pub const GET_RICH_SCHEMA_BY_ID: &str = "300";
pub const GET_RICH_SCHEMA_BY_METADATA: &str = "301";

pub const REQUESTS: [&str; 35] = [
    NODE,
    NYM,
    GET_TXN,
    ATTRIB,
    SCHEMA,
    CRED_DEF,
    GET_ATTR,
    GET_NYM,
    GET_SCHEMA,
    GET_CRED_DEF,
    POOL_UPGRADE,
    POOL_RESTART,
    POOL_CONFIG,
    REVOC_REG_DEF,
    REVOC_REG_ENTRY,
    GET_REVOC_REG_DEF,
    GET_REVOC_REG,
    GET_REVOC_REG_DELTA,
    GET_VALIDATOR_INFO,
    AUTH_RULE,
    TXN_AUTHR_AGRMT,
    TXN_AUTHR_AGRMT_AML,
    GET_TXN_AUTHR_AGRMT,
    GET_TXN_AUTHR_AGRMT_AML,
    DISABLE_ALL_TXN_AUTHR_AGRMTS,
    LEDGERS_FREEZE,
    GET_FROZEN_LEDGERS,
    FLAG,
    GET_FLAG,
    RICH_SCHEMA_CTX,
    RICH_SCHEMA,
    RICH_SCHEMA_ENCODING,
    RICH_SCHEMA_MAPPING,
    RICH_SCHEMA_CRED_DEF,
    RICH_SCHEMA_PRES_DEF,
];

// likely matches REQUESTS_FOR_STATE_PROOFS
pub const READ_REQUESTS: [&str; 14] = [
    GET_NYM,
    GET_TXN_AUTHR_AGRMT,
    GET_TXN_AUTHR_AGRMT_AML,
    GET_SCHEMA,
    GET_CRED_DEF,
    GET_ATTR,
    GET_REVOC_REG,
    GET_REVOC_REG_DEF,
    GET_REVOC_REG_DELTA,
    GET_AUTH_RULE,
    GET_TXN,
    GET_FLAG,
    GET_RICH_SCHEMA_BY_ID,
    GET_RICH_SCHEMA_BY_METADATA,
];

pub const ROLE_TRUSTEE: usize = 0;
pub const ROLE_STEWARD: usize = 2;
pub const ROLE_ENDORSER: usize = 101;
pub const ROLE_NETWORK_MONITOR: usize = 201;

pub const RS_SCHEMA_TYPE_VALUE: &str = "sch";
pub const RS_ENCODING_TYPE_VALUE: &str = "enc";
pub const RS_CONTEXT_TYPE_VALUE: &str = "ctx";
pub const RS_MAPPING_TYPE_VALUE: &str = "map";
pub const RS_CRED_DEF_TYPE_VALUE: &str = "cdf";
pub const RS_PRES_DEF_TYPE_VALUE: &str = "pdf";

// Method/version of self-certification
/// No (enforced) self-certification
pub const CERT_DEFAULT: i32 = 0;
/// Legacy self-certification
pub const CERT_DID_SOV: i32 = 1;
/// Self-certification based on did:indy method spec
pub const CERT_DID_INDY: i32 = 2;

pub const SELF_CERT_VERSIONS: [i32; 3] = [CERT_DEFAULT, CERT_DID_SOV, CERT_DID_INDY];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum LedgerRole {
    Trustee,
    Steward,
    Endorser,
    NetworkMonitor,
    Custom(usize),
}

impl LedgerRole {
    pub fn to_usize(self) -> usize {
        match self {
            Self::Trustee => ROLE_TRUSTEE,
            Self::Steward => ROLE_STEWARD,
            Self::Endorser => ROLE_ENDORSER,
            Self::NetworkMonitor => ROLE_NETWORK_MONITOR,
            Self::Custom(val) => val,
        }
    }

    pub fn to_code(self) -> String {
        self.to_usize().to_string()
    }
}

impl fmt::Display for LedgerRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident = match self {
            Self::Trustee => "TRUSTEE",
            Self::Steward => "STEWARD",
            Self::Endorser => "ENDORSER",
            Self::NetworkMonitor => "NETWORK_MONITOR",
            Self::Custom(role) => return write!(f, "{}", role),
        };
        f.write_str(ident)
    }
}

impl FromStr for LedgerRole {
    type Err = VdrError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "TRUSTEE" => Ok(Self::Trustee),
            "STEWARD" => Ok(Self::Steward),
            "TRUST_ANCHOR" | "ENDORSER" => Ok(Self::Endorser),
            "NETWORK_MONITOR" => Ok(Self::NetworkMonitor),
            _ => {
                if let Ok(role) = value.parse::<usize>() {
                    Ok(Self::from(role))
                } else {
                    Err(input_err(format!("Invalid ledger role: {value}")))
                }
            }
        }
    }
}

impl From<usize> for LedgerRole {
    fn from(value: usize) -> Self {
        match value {
            ROLE_TRUSTEE => Self::Trustee,
            ROLE_STEWARD => Self::Steward,
            ROLE_ENDORSER => Self::Endorser,
            ROLE_NETWORK_MONITOR => Self::NetworkMonitor,
            other => Self::Custom(other),
        }
    }
}

impl From<LedgerRole> for usize {
    fn from(value: LedgerRole) -> Self {
        value.to_usize()
    }
}

impl<'d> Deserialize<'d> for LedgerRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        struct Vis;

        impl<'v> Visitor<'v> for Vis {
            type Value = LedgerRole;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expected a ledger role identifier")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                LedgerRole::from_str(v).map_err(|e| E::custom(e.to_string()))
            }
        }

        deserializer.deserialize_any(Vis)
    }
}

impl Serialize for LedgerRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.to_usize())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateRole {
    Reset,
    Set(LedgerRole),
}

impl FromStr for UpdateRole {
    type Err = VdrError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "" => Ok(Self::Reset),
            _ => LedgerRole::from_str(value).map(Self::Set),
        }
    }
}

impl<'d> Deserialize<'d> for UpdateRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        struct Vis;

        impl<'v> Visitor<'v> for Vis {
            type Value = UpdateRole;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expected a ledger role identifier")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(UpdateRole::Reset)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                LedgerRole::from_str(v)
                    .map_err(|e| E::custom(e.to_string()))
                    .map(UpdateRole::Set)
            }
        }

        deserializer.deserialize_any(Vis)
    }
}

impl Serialize for UpdateRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Reset => serializer.serialize_none(),
            Self::Set(role) => serializer.collect_str(&role.to_usize()),
        }
    }
}

pub fn txn_name_to_code(txn: &str) -> Option<&str> {
    if REQUESTS.contains(&txn) {
        return Some(txn);
    }

    match txn {
        "NODE" => Some(NODE),
        "NYM" => Some(NYM),
        "GET_TXN" => Some(GET_TXN),
        "ATTRIB" => Some(ATTRIB),
        "SCHEMA" => Some(SCHEMA),
        "CRED_DEF" | "CLAIM_DEF" => Some(CRED_DEF),
        "GET_ATTR" => Some(GET_ATTR),
        "GET_NYM" => Some(GET_NYM),
        "GET_SCHEMA" => Some(GET_SCHEMA),
        "GET_CRED_DEF" => Some(GET_CRED_DEF),
        "POOL_UPGRADE" => Some(POOL_UPGRADE),
        "POOL_RESTART" => Some(POOL_RESTART),
        "POOL_CONFIG" => Some(POOL_CONFIG),
        "REVOC_REG_DEF" => Some(REVOC_REG_DEF),
        "REVOC_REG_ENTRY" => Some(REVOC_REG_ENTRY),
        "GET_REVOC_REG_DEF" => Some(GET_REVOC_REG_DEF),
        "GET_REVOC_REG" => Some(GET_REVOC_REG),
        "GET_REVOC_REG_DELTA" => Some(GET_REVOC_REG_DELTA),
        "GET_VALIDATOR_INFO" => Some(GET_VALIDATOR_INFO),
        "AUTH_RULE" => Some(AUTH_RULE),
        "TXN_AUTHR_AGRMT" => Some(TXN_AUTHR_AGRMT),
        "TXN_AUTHR_AGRMT_AML" => Some(TXN_AUTHR_AGRMT_AML),
        "GET_TXN_AUTHR_AGRMT" => Some(GET_TXN_AUTHR_AGRMT),
        "GET_TXN_AUTHR_AGRMT_AML" => Some(GET_TXN_AUTHR_AGRMT_AML),
        "DISABLE_ALL_TXN_AUTHR_AGRMTS" => Some(DISABLE_ALL_TXN_AUTHR_AGRMTS),
        "LEDGERS_FREEZE" => Some(LEDGERS_FREEZE),
        "GET_FROZEN_LEDGERS" => Some(GET_FROZEN_LEDGERS),
        "FLAG" => Some(FLAG),
        "GET_FLAG" => Some(GET_FLAG),
        val => Some(val),
    }
}
