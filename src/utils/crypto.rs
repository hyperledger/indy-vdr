use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::Signature;
use ed25519_dalek::{Keypair, PublicKey};
use rand::rngs::OsRng;

use crate::utils::error::prelude::*;

pub fn gen_keypair() -> LedgerResult<Keypair> {
    let mut csprng = OsRng::new().to_result(
        LedgerErrorKind::InvalidState,
        "Error creating random number generator",
    )?;
    Ok(Keypair::generate(&mut csprng))
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
