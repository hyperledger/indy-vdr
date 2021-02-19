use crate::utils::constants::*;
use crate::utils::crypto::Identity;
use crate::utils::pool::TestPool;
use indy_vdr::ledger::RequestBuilder;
use indy_vdr::pool::ProtocolVersion;
use indy_vdr::utils::did::DidValue;
use rstest::*;

#[fixture]
pub fn trustee() -> Identity {
    Identity::new(Some(TRUSTEE_SEED))
}

#[fixture]
pub fn trustee_did() -> DidValue {
    DidValue(String::from(TRUSTEE_DID))
}

#[fixture]
pub fn fq_trustee_did() -> DidValue {
    DidValue(String::from(TRUSTEE_DID_FQ))
}

#[fixture]
pub fn steward_did() -> DidValue {
    DidValue(String::from(STEWARD_DID))
}

#[fixture]
pub fn my_did() -> DidValue {
    DidValue(String::from(MY1_DID))
}

#[fixture]
pub fn my_verkey() -> DidValue {
    DidValue(String::from(MY1_VERKEY))
}

#[fixture]
pub fn fq_my_did() -> DidValue {
    DidValue(String::from(MY1_DID_FQ))
}

#[fixture]
pub fn identity() -> Identity {
    Identity::new(None)
}

#[fixture]
pub fn pool() -> TestPool {
    TestPool::new()
}

#[fixture]
pub fn request_builder() -> RequestBuilder {
    RequestBuilder::new(ProtocolVersion::Node1_4)
}

#[fixture]
pub fn timestamp() -> u64 {
    12345
}
