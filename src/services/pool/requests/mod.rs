#[allow(unused_imports)]
use super::networker;
use super::types;

use crate::utils::base58::FromBase58;
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

pub mod catchup;
pub mod status;

fn get_f(cnt: usize) -> usize {
    if cnt < 4 {
        return 0;
    }
    (cnt - 1) / 3
}

fn check_cons_proofs(
    mt: &MerkleTree,
    cons_proofs: &Vec<String>,
    target_mt_root: &Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<()> {
    let mut bytes_proofs: Vec<Vec<u8>> = Vec::new();

    for cons_proof in cons_proofs {
        let cons_proof: &String = cons_proof;

        bytes_proofs.push(
            cons_proof.from_base58().to_result(
                LedgerErrorKind::InvalidStructure,
                "Can't decode node consistency proof",
            )?, // FIXME: review kind
        );
    }

    if !mt.consistency_proof(target_mt_root, target_mt_size, &bytes_proofs)? {
        return Err(err_msg(
            LedgerErrorKind::InvalidState,
            "Consistency proof verification failed",
        )); // FIXME: review kind
    }

    Ok(())
}
