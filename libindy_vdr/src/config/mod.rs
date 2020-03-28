pub(crate) mod constants;
pub(crate) mod types;

pub use types::PoolConfig;

/// Library package version
pub static LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
