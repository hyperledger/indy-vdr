use indy_vdr::pool::PreparedRequest;
use indy_vdr::utils::did::{generate_did, DidValue};
use indy_vdr::utils::keys::{PrivateKey, VerKey};

use crate::utils::constants::*;

pub struct Identity {
    pub did: DidValue,
    pub verkey: String,
    _public_key: VerKey,
    private_key: PrivateKey,
}

impl Identity {
    pub fn trustee() -> Identity {
        Identity::new(Some(TRUSTEE_SEED))
    }

    pub fn new(seed: Option<[u8; 32]>) -> Identity {
        let (short_did, private_key, public_key) =
            generate_did(seed.as_ref().map(|s| &s[..])).unwrap();

        let verkey = public_key.as_base58().unwrap().to_string();

        Identity {
            did: DidValue((*short_did).to_owned()),
            verkey,
            _public_key: public_key,
            private_key,
        }
    }

    fn _generate_signature(&self, request: &mut PreparedRequest) -> Vec<u8> {
        let signature_input = request.get_signature_input().unwrap();
        self.private_key.sign(signature_input.as_bytes()).unwrap()
    }

    pub fn sign_request(&self, request: &mut PreparedRequest) {
        let signature = self._generate_signature(request);
        request.set_signature(signature.as_slice()).unwrap();
    }

    pub fn multi_sign_request(&self, request: &mut PreparedRequest) {
        let signature = self._generate_signature(request);
        request
            .set_multi_signature(&self.did, signature.as_slice())
            .unwrap();
    }
}
