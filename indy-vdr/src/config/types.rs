pub use crate::pool::ProtocolVersion;

use crate::utils::validation::Validatable;

pub const DEFAULT_ACK_TIMEOUT: i64 = 20;
pub const DEFAULT_REPLY_TIMEOUT: i64 = 60;
pub const DEFAULT_CONN_ACTIVE_TIMEOUT: i64 = 5;
pub const DEFAULT_CONN_REQUEST_LIMIT: usize = 5;
pub const DEFAULT_REQUEST_READ_NODES: usize = 2;
pub const DEFAULT_FRESHNESS_TIMEOUT: u64 = 300;
pub const DEFAULT_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Node1_4;
pub const DEFAULT_GENERATOR: &str = "3LHpUjiyFC2q2hD7MnwwNmVXiuaFbQx2XkAFJWzswCjgN1utjsCeLzHsKk1nJvFEaS4fcrUmVAkdhtPCYbrVyATZcmzwJReTcJqwqBCPTmTQ9uWPwz6rEncKb2pYYYFcdHa8N17HzVyTqKfgPi4X9pMetfT3A5xCHq54R2pDNYWVLDX";

#[derive(Debug, Copy, Clone)]
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
    fn validate(&self) -> Result<(), String> {
        if self.freshness_threshold == 0 {
            return Err(String::from("`freshness_threshold` must be greater than 0"));
        }
        if self.ack_timeout <= 0 {
            return Err(String::from("`ack_timeout` must be greater than 0"));
        }
        if self.reply_timeout <= 0 {
            return Err(String::from("`reply_timeout` must be greater than 0"));
        }
        if self.conn_request_limit == 0 {
            return Err(String::from("`conn_request_limit` must be greater than 0"));
        }
        if self.conn_active_timeout <= 0 {
            return Err(String::from("`conn_active_timeout` must be greater than 0"));
        }
        if self.request_read_nodes == 0 {
            return Err(String::from("`request_read_nodes` must be greater than 0"));
        }
        Ok(())
    }
}

impl PoolConfig {
    fn default_freshness_threshold() -> u64 {
        DEFAULT_FRESHNESS_TIMEOUT
    }

    fn default_protocol_version() -> ProtocolVersion {
        DEFAULT_PROTOCOL_VERSION
    }

    fn default_ack_timeout() -> i64 {
        DEFAULT_ACK_TIMEOUT
    }

    fn default_reply_timeout() -> i64 {
        DEFAULT_REPLY_TIMEOUT
    }

    fn default_conn_request_limit() -> usize {
        DEFAULT_CONN_REQUEST_LIMIT
    }

    fn default_conn_active_timeout() -> i64 {
        DEFAULT_CONN_ACTIVE_TIMEOUT
    }

    fn default_request_read_nodes() -> usize {
        DEFAULT_REQUEST_READ_NODES
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
