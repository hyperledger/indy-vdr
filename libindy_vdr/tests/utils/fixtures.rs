use indy_utils::base58;
use rand::{thread_rng, Rng};

use crate::utils::constants::*;
use crate::utils::crypto::Identity;
use crate::utils::pool::TestPool;
use indy_vdr::ledger::RequestBuilder;
use indy_vdr::pool::ProtocolVersion;
use indy_vdr::utils::did::DidValue;
use rstest::*;

#[fixture]
pub fn trustee() -> Identity {
    Identity::new(Some(TRUSTEE_SEED), None)
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
// did:sov self-certified DID
pub fn identity() -> Identity {
    Identity::new(None, None)
}

#[fixture]
// did:indy self-certified DID
pub fn identity_v2() -> Identity {
    Identity::new(None, Some(2))
}

#[fixture]
// Generate a random DID without relation to the pubkey
pub fn non_self_cert_identity() -> Identity {
    let mut id = Identity::new(None, None);
    let mut rng = thread_rng();
    let rand_arr: [u8; 16] = rng.gen();
    let did = base58::encode(rand_arr);
    id.did = DidValue(did);
    id
}

#[fixture]
pub fn diddoc_content() -> serde_json::Value {
    serde_json::json!({
    "service": [
      {
        "id": "did:indy:sovrin:123456#did-communication",
        "type": "did-communication",
        "serviceEndpoint": "https://example.com",
        "recipientKeys": [ "#verkey" ],
        "routingKeys": [ ]
      }
    ]
    })
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
