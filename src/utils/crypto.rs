use curve25519_dalek::edwards::CompressedEdwardsY;
pub use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use rand::rngs::OsRng;

use crate::utils::error::prelude::*;

pub const CRYPTO_TYPE_ED25519: &str = "ed25519";
pub const DEFAULT_CRYPTO_TYPE: &str = CRYPTO_TYPE_ED25519;

pub fn gen_keypair() -> LedgerResult<Keypair> {
    let mut csprng = OsRng::new().to_result(
        LedgerErrorKind::InvalidState,
        "Error creating random number generator",
    )?;
    Ok(Keypair::generate(&mut csprng))
}

pub fn import_verkey(vk: Vec<u8>) -> LedgerResult<PublicKey> {
    PublicKey::from_bytes(&vk).to_result(LedgerErrorKind::InvalidStructure, "Error decoding verkey")
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
