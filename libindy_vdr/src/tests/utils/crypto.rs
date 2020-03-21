use crate::common::did::DidValue;
use crate::ledger::PreparedRequest;
use crate::tests::utils::constants::TRUSTEE_SEED;
use crate::utils::base58::ToBase58;
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

        let verkey = public_key[..].to_base58();
        let did = public_key[0..16].to_vec().to_base58();

        Identity {
            did: DidValue(did),
            verkey,
            _public_key: public_key,
            private_key,
        }
    }

    pub fn sign_request(&self, request: &mut PreparedRequest) {
        let signature_input = request.get_signature_input().unwrap();

        let ed25519 = Ed25519Sha512::new();
        let signature = ed25519
            .sign(signature_input.as_bytes(), &self.private_key)
            .unwrap();
        request.set_signature(signature.as_slice()).unwrap();
    }
}
