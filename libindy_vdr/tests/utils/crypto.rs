use indy_vdr::common::did::DidValue;
use indy_vdr::pool::PreparedRequest;
use vdr_shared::base58;

use crate::utils::constants::*;

use ursa::keys::{KeyGenOption, PrivateKey, PublicKey};
use ursa::signatures::ed25519::Ed25519Sha512;
use ursa::signatures::SignatureScheme;

pub struct Identity {
    pub did: DidValue,
    pub verkey: String,
    _public_key: PublicKey,
    private_key: PrivateKey,
}

impl Identity {
    pub fn trustee() -> Identity {
        Identity::new(Some(TRUSTEE_SEED))
    }

    pub fn new(seed: Option<[u8; 64]>) -> Identity {
        let ed25519 = Ed25519Sha512::new();

        let seed = seed.map(|seed_| KeyGenOption::FromSecretKey(PrivateKey(seed_.to_vec()))); // TODO: FIXME MUST BE SEED but seems Ursa provides different derivation

        let (public_key, private_key) = ed25519.keypair(seed).unwrap();

        let verkey = base58::encode(&public_key);
        let did = base58::encode(&public_key[0..16]);

        Identity {
            did: DidValue(did),
            verkey,
            _public_key: public_key,
            private_key,
        }
    }

    fn _generate_signature(&self, request: &mut PreparedRequest) -> Vec<u8> {
        let signature_input = request.get_signature_input().unwrap();
        let ed25519 = Ed25519Sha512::new();
        ed25519
            .sign(signature_input.as_bytes(), &self.private_key)
            .unwrap()
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
