#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod macros;
/// Trait for qualifiable identifier types, having an optional prefix and method
#[macro_use]
pub mod qualifier;
/// Trait and error definition for validatable data types
#[macro_use]
pub mod validation;

pub mod base58;
pub mod crypto;
pub mod environment;
pub mod hash;

#[macro_use]
pub mod test;
