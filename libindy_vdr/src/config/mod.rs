pub mod constants;
pub mod types;

pub(crate) use types::PoolConfig;

pub static LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
