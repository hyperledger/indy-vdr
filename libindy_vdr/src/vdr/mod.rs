/// Indy VDR implementation for multiple ledgers and DID resolution/derefrence support
mod vdr;

pub mod utils;

pub use self::vdr::{RunnerVdr, Vdr};
