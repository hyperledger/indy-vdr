pub use crate::pool::ProtocolVersion;

use crate::common::error::prelude::*;
use crate::utils::validation::Validatable;

use super::constants;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub protocol_version: ProtocolVersion,
    pub freshness_threshold: u64,
    pub ack_timeout: i64,
    pub reply_timeout: i64,
    pub conn_request_limit: usize,
    pub conn_active_timeout: i64,
    pub request_read_nodes: usize,
}

impl Validatable for PoolConfig {
    fn validate(&self) -> LedgerResult<()> {
        if self.freshness_threshold == 0 {
            return Err(input_err("`freshness_threshold` must be greater than 0"));
        }
        if self.ack_timeout <= 0 {
            return Err(input_err("`ack_timeout` must be greater than 0"));
        }
        if self.reply_timeout <= 0 {
            return Err(input_err("`reply_timeout` must be greater than 0"));
        }
        if self.conn_request_limit == 0 {
            return Err(input_err("`conn_request_limit` must be greater than 0"));
        }
        if self.conn_active_timeout <= 0 {
            return Err(input_err("`conn_active_timeout` must be greater than 0"));
        }
        if self.request_read_nodes == 0 {
            return Err(input_err("`request_read_nodes` must be greater than 0"));
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
