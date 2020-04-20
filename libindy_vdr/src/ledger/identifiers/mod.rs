/// Credential definition identifiers
pub mod cred_def;
/// Revocation registry identifiers
pub mod rev_reg;
#[cfg(feature = "rich_schema")]
/// Rich schema identifiers
pub mod rich_schema;
/// V1 schema identifiers
pub mod schema;

pub(crate) use crate::common::did;

/// The standard delimiter used in identifier strings
pub const DELIMITER: &'static str = ":";
