#[allow(unused_imports)]
use serde_json::{self, Value as SJsonValue};

use super::networker;
use super::state_proof;
use super::types;

use crate::utils::base58::FromBase58;
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

pub mod catchup;
pub mod single;
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

fn serialize_message(message: &types::Message) -> LedgerResult<(String, String)> {
    let req_id = message.request_id().unwrap_or("".to_owned());
    let req_json = serde_json::to_string(&message).map_err(|err| {
        err_msg(
            LedgerErrorKind::InvalidState,
            format!("Cannot serialize request: {:?}", err),
        )
    })?;
    Ok((req_id, req_json))
}

fn get_msg_result_without_state_proof(msg: &str) -> LedgerResult<(SJsonValue, SJsonValue)> {
    let msg = serde_json::from_str::<SJsonValue>(msg).to_result(
        LedgerErrorKind::InvalidStructure,
        "Response is malformed json",
    )?;

    let msg_result = msg["result"].clone();

    let mut msg_result_without_proof: SJsonValue = msg_result.clone();
    msg_result_without_proof
        .as_object_mut()
        .map(|obj| obj.remove("state_proof"));

    if msg_result_without_proof["data"].is_object() {
        msg_result_without_proof["data"]
            .as_object_mut()
            .map(|obj| obj.remove("stateProofFrom"));
    }

    Ok((msg_result, msg_result_without_proof))
}
