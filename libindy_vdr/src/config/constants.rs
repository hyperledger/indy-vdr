use crate::pool::ProtocolVersion;

pub const DEFAULT_ACK_TIMEOUT: i64 = 20;
pub const DEFAULT_REPLY_TIMEOUT: i64 = 60;
pub const DEFAULT_CONN_ACTIVE_TIMEOUT: i64 = 5;
pub const DEFAULT_CONN_REQUEST_LIMIT: usize = 5;
pub const DEFAULT_REQUEST_READ_NODES: usize = 2;
pub const DEFAULT_FRESHNESS_TIMEOUT: u64 = 300;
pub const DEFAULT_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Node1_4;
pub const DEFAULT_GENERATOR: &str = "3LHpUjiyFC2q2hD7MnwwNmVXiuaFbQx2XkAFJWzswCjgN1utjsCeLzHsKk1nJvFEaS4fcrUmVAkdhtPCYbrVyATZcmzwJReTcJqwqBCPTmTQ9uWPwz6rEncKb2pYYYFcdHa8N17HzVyTqKfgPi4X9pMetfT3A5xCHq54R2pDNYWVLDX";
