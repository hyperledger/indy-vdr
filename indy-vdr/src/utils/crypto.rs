use curve25519_dalek::edwards::CompressedEdwardsY;
pub use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey, SecretKey};
use rand::rngs::OsRng;

use crate::common::error::prelude::*;

pub const CRYPTO_TYPE_ED25519: &str = "ed25519";
pub const DEFAULT_CRYPTO_TYPE: &str = CRYPTO_TYPE_ED25519;

#[allow(dead_code)]
pub fn gen_keypair() -> LedgerResult<Keypair> {
    let mut csprng = OsRng::new().to_result(
        LedgerErrorKind::InvalidState,
        "Error creating random number generator",
    )?;
    Ok(Keypair::generate(&mut csprng))
}

pub fn import_verkey(vk: &[u8]) -> LedgerResult<PublicKey> {
    PublicKey::from_bytes(&vk).to_result(LedgerErrorKind::InvalidStructure, "Error decoding verkey")
}

pub fn import_keypair(secret: &[u8]) -> LedgerResult<Keypair> {
    let sk = SecretKey::from_bytes(&secret)
        .to_result(LedgerErrorKind::InvalidStructure, "Error decoding key")?;
    let pk: PublicKey = (&sk).into();
    Ok(Keypair {
        secret: sk,
        public: pk,
    })
}

pub fn vk_to_curve25519(vk: PublicKey) -> LedgerResult<Vec<u8>> {
    let edw = unwrap_opt_or_return!(
        CompressedEdwardsY::from_slice(&vk.to_bytes()).decompress(),
        Err(err_msg(
            LedgerErrorKind::InvalidState,
            "Error loading public key"
        ))
    );
    Ok(edw.to_montgomery().to_bytes().to_vec())
}

#[allow(dead_code)]
pub fn sk_to_curve25519(sk: SecretKey) -> LedgerResult<Vec<u8>> {
    let edw = unwrap_opt_or_return!(
        CompressedEdwardsY::from_slice(&sk.to_bytes()).decompress(),
        Err(err_msg(
            LedgerErrorKind::InvalidState,
            "Error loading secret key"
        ))
    );
    Ok(edw.to_montgomery().to_bytes().to_vec())
}

pub fn sign_message(keypair: Keypair, message: &[u8]) -> Vec<u8> {
    let signature = keypair.sign(message);
    signature.to_bytes().to_vec()
}
