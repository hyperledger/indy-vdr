use crate::pool::ProtocolVersion;
use crate::utils::{Validatable, ValidationError};

use super::constants;

/// Configuration settings for managing validator pool communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// The protocol version used in pool communication
    #[serde(default = "PoolConfig::default_protocol_version")]
    pub protocol_version: ProtocolVersion,
    /// The freshness threshold to accept when validating state proofs
    #[serde(default = "PoolConfig::default_freshness_threshold")]
    pub freshness_threshold: u64,
    /// The timeout to use when waiting for responses from multiple nodes
    #[serde(default = "PoolConfig::default_ack_timeout")]
    pub ack_timeout: i64,
    /// The timeout for pool read and write transactions
    #[serde(default = "PoolConfig::default_reply_timeout")]
    pub reply_timeout: i64,
    /// The maximum number of requests to process before creating a new pool connection
    #[serde(default = "PoolConfig::default_conn_request_limit")]
    pub conn_request_limit: usize,
    /// The timeout before an active pool connection will stop accepting new requests
    #[serde(default = "PoolConfig::default_conn_active_timeout")]
    pub conn_active_timeout: i64,
    /// The initial number of nodes to send ledger read requests
    #[serde(default = "PoolConfig::default_request_read_nodes")]
    pub request_read_nodes: usize,
    /// The socks proxy host name and port for ZMQ (example: proxy1.intranet.company.com:1080)
    #[serde(default = "PoolConfig::default_socks_proxy")]
    pub socks_proxy: Option<String>,
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
    /// The default freshness threshold to accept when validating state proofs
    pub fn default_freshness_threshold() -> u64 {
        constants::DEFAULT_FRESHNESS_TIMEOUT
    }

    /// The default protocol version for pool communication
    pub fn default_protocol_version() -> ProtocolVersion {
        constants::DEFAULT_PROTOCOL_VERSION
    }

    /// The default timeout when waiting for responses from multiple nodes
    pub fn default_ack_timeout() -> i64 {
        constants::DEFAULT_ACK_TIMEOUT
    }

    /// The default timeout for pool read and write transactions
    pub fn default_reply_timeout() -> i64 {
        constants::DEFAULT_REPLY_TIMEOUT
    }

    /// The default maximum number of requests to process before creating a new pool connection
    pub fn default_conn_request_limit() -> usize {
        constants::DEFAULT_CONN_REQUEST_LIMIT
    }

    /// The default timeout before discarding an active pool connection
    pub fn default_conn_active_timeout() -> i64 {
        constants::DEFAULT_CONN_ACTIVE_TIMEOUT
    }

    /// The default initial number of nodes to send ledger read requests
    pub fn default_request_read_nodes() -> usize {
        constants::DEFAULT_REQUEST_READ_NODES
    }

    /// The default socks proxy is empty / unset
    pub fn default_socks_proxy() -> Option<String> {
        None
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
            socks_proxy: Self::default_socks_proxy(),
        }
    }
}
