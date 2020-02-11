use ursa::keys::PublicKey;
use ursa::signatures::ed25519::Ed25519Sha512;

use crate::common::error::prelude::*;

pub const CRYPTO_TYPE_ED25519: &str = "ed25519";
pub const DEFAULT_CRYPTO_TYPE: &str = CRYPTO_TYPE_ED25519;

pub fn vk_to_curve25519(vk: &[u8]) -> VdrResult<Vec<u8>> {
    let vk = PublicKey(vk.to_vec());
    Ok(Ed25519Sha512::ver_key_to_key_exchange(&vk)
        .map_err(|err| input_err(format!("Error converting to curve25519 key: {}", err)))?
        .0
        .clone())
}
