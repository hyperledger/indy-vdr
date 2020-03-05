pub use crate::pool::ProtocolVersion;

use crate::utils::validation::{Validatable, ValidationError};

use super::constants;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    #[serde(default = "PoolConfig::default_protocol_version")]
    pub protocol_version: ProtocolVersion,
    #[serde(default = "PoolConfig::default_freshness_threshold")]
    pub freshness_threshold: u64,
    #[serde(default = "PoolConfig::default_ack_timeout")]
    pub ack_timeout: i64,
    #[serde(default = "PoolConfig::default_reply_timeout")]
    pub reply_timeout: i64,
    #[serde(default = "PoolConfig::default_conn_request_limit")]
    pub conn_request_limit: usize,
    #[serde(default = "PoolConfig::default_conn_active_timeout")]
    pub conn_active_timeout: i64,
    #[serde(default = "PoolConfig::default_request_read_nodes")]
    pub request_read_nodes: usize,
}

impl Validatable for PoolConfig {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.freshness_threshold == 0 {
            return Err(invalid!("`freshness_threshold` must be greater than 0"));
        }
        if self.ack_timeout <= 0 {
            return Err(invalid!("`ack_timeout` must be greater than 0"));
        }
        if self.reply_timeout <= 0 {
            return Err(invalid!("`reply_timeout` must be greater than 0"));
        }
        if self.conn_request_limit == 0 {
            return Err(invalid!("`conn_request_limit` must be greater than 0"));
        }
        if self.conn_active_timeout <= 0 {
            return Err(invalid!("`conn_active_timeout` must be greater than 0"));
        }
        if self.request_read_nodes == 0 {
            return Err(invalid!("`request_read_nodes` must be greater than 0"));
        }
        Ok(())
    }
}

impl PoolConfig {
    fn default_freshness_threshold() -> u64 {
        constants::DEFAULT_FRESHNESS_TIMEOUT
    }

    fn default_protocol_version() -> ProtocolVersion {
        constants::DEFAULT_PROTOCOL_VERSION
    }

    fn default_ack_timeout() -> i64 {
        constants::DEFAULT_ACK_TIMEOUT
    }

    fn default_reply_timeout() -> i64 {
        constants::DEFAULT_REPLY_TIMEOUT
    }

    fn default_conn_request_limit() -> usize {
        constants::DEFAULT_CONN_REQUEST_LIMIT
    }

    fn default_conn_active_timeout() -> i64 {
        constants::DEFAULT_CONN_ACTIVE_TIMEOUT
    }

    fn default_request_read_nodes() -> usize {
        constants::DEFAULT_REQUEST_READ_NODES
    }
}

impl Default for PoolConfig {
    fn default() -> PoolConfig {
        PoolConfig {
            protocol_version: Self::default_protocol_version(),
            freshness_threshold: Self::default_freshness_threshold(),
            ack_timeout: Self::default_ack_timeout(),
            reply_timeout: Self::default_reply_timeout(),
            conn_request_limit: Self::default_conn_request_limit(),
            conn_active_timeout: Self::default_conn_active_timeout(),
            request_read_nodes: Self::default_request_read_nodes(),
        }
    }
}
