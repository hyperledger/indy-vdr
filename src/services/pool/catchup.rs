use std::collections::HashMap;
use std::collections::HashSet;

use failure::Context;
use serde_json;

use super::types::{CatchupReq, LedgerStatus, Message};
use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::{FromBase58, ToBase58};
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

pub enum CatchupProgress {
    ConsensusNotReachable,
    ShouldBeStarted(
        Vec<u8>, //target_mt_root
        usize,   //target_mt_size
    ),
    NotNeeded,
    InProgress,
    Timeout,
}

pub fn build_ledger_status_req(
    merkle: &MerkleTree,
    protocol_version: ProtocolVersion,
) -> LedgerResult<(String, String)> {
    let lr = LedgerStatus {
        txnSeqNo: merkle.count(),
        merkleRoot: merkle.root_hash().as_slice().to_base58(),
        ledgerId: 0,
        ppSeqNo: None,
        viewNo: None,
        protocolVersion: Some(protocol_version as usize),
    };
    let req_id = lr.merkleRoot.clone();
    let req_json = serde_json::to_string(&super::types::Message::LedgerStatus(lr)).to_result(
        LedgerErrorKind::InvalidState,
        "Cannot serialize LedgerStatus",
    )?;
    Ok((req_id, req_json))
}

pub fn build_catchup_req(
    merkle: &MerkleTree,
    target_mt_size: usize,
) -> LedgerResult<Option<(String, String)>> {
    if merkle.count() >= target_mt_size {
        warn!("No transactions to catch up!");
        return Ok(None);
    }
    let seq_no_start = merkle.count() + 1;
    let seq_no_end = target_mt_size;

    let cr = CatchupReq {
        ledgerId: 0,
        seqNoStart: seq_no_start,
        seqNoEnd: seq_no_end,
        catchupTill: target_mt_size,
    };

    let req_id = format!("{}{}", seq_no_start, seq_no_end);

    let req_json = serde_json::to_string(&Message::CatchupReq(cr)).to_result(
        LedgerErrorKind::InvalidState,
        "Cannot serialize CatchupRequest",
    )?;

    trace!("catchup_req msg: {:?}", req_json);
    Ok(Some((req_id, req_json)))
}

pub fn check_nodes_responses_on_status(
    nodes_votes: &HashMap<(String, usize, Option<Vec<String>>), HashSet<String>>,
    merkle_tree: &MerkleTree,
    node_cnt: usize,
    f: usize,
) -> LedgerResult<CatchupProgress> {
    let (votes, timeout_votes): (
        HashMap<&(String, usize, Option<Vec<String>>), usize>,
        HashMap<&(String, usize, Option<Vec<String>>), usize>,
    ) = nodes_votes
        .iter()
        .map(|(key, val)| (key, val.len()))
        .partition(|((key, _, _), _)| key != "timeout");

    let most_popular_not_timeout = votes.iter().max_by_key(|entry| entry.1);

    let timeout_votes = timeout_votes.iter().last();

    if let Some((most_popular_not_timeout_vote, votes_cnt)) = most_popular_not_timeout {
        if *votes_cnt == f + 1 {
            _try_to_catch_up(most_popular_not_timeout_vote, merkle_tree)
        } else {
            _if_consensus_reachable(nodes_votes, node_cnt, *votes_cnt, f)
        }
    } else if let Some((_, votes_cnt)) = timeout_votes {
        if *votes_cnt == node_cnt - f {
            Ok(CatchupProgress::Timeout)
        } else {
            _if_consensus_reachable(nodes_votes, node_cnt, *votes_cnt, f)
        }
    } else {
        Ok(CatchupProgress::InProgress)
    }
}

fn _if_consensus_reachable(
    nodes_votes: &HashMap<(String, usize, Option<Vec<String>>), HashSet<String>>,
    node_cnt: usize,
    votes_cnt: usize,
    f: usize,
) -> LedgerResult<CatchupProgress> {
    let reps_cnt: usize = nodes_votes.values().map(HashSet::len).sum();
    let positive_votes_cnt = votes_cnt + (node_cnt - reps_cnt);
    let is_consensus_not_reachable = positive_votes_cnt < node_cnt - f;
    if is_consensus_not_reachable {
        Ok(CatchupProgress::ConsensusNotReachable)
    } else {
        Ok(CatchupProgress::InProgress)
    }
}

fn _try_to_catch_up(
    ledger_status: &(String, usize, Option<Vec<String>>),
    merkle_tree: &MerkleTree,
) -> LedgerResult<CatchupProgress> {
    let &(ref target_mt_root, target_mt_size, ref hashes) = ledger_status;
    let cur_mt_size = merkle_tree.count();
    let cur_mt_hash = merkle_tree.root_hash().to_base58();

    if target_mt_size == cur_mt_size {
        if cur_mt_hash.eq(target_mt_root) {
            Ok(CatchupProgress::NotNeeded)
        } else {
            Err(err_msg(
                LedgerErrorKind::InvalidState,
                "Ledger merkle tree is not acceptable for current tree.",
            ))
        }
    } else if target_mt_size > cur_mt_size {
        let target_mt_root = target_mt_root
            .from_base58()
            .map_err(Context::new)
            .to_result(
                LedgerErrorKind::InvalidStructure,
                "Can't parse target MerkleTree hash from nodes responses",
            )?; // FIXME: review kind

        match *hashes {
            None => (),
            Some(ref hashes) => {
                check_cons_proofs(merkle_tree, hashes, &target_mt_root, target_mt_size)?
            }
        };

        Ok(CatchupProgress::ShouldBeStarted(
            target_mt_root,
            target_mt_size,
        ))
    } else {
        Err(err_msg(
            LedgerErrorKind::InvalidState,
            "Local merkle tree greater than mt from ledger",
        ))
    }
}

pub fn check_cons_proofs(
    mt: &MerkleTree,
    cons_proofs: &Vec<String>,
    target_mt_root: &Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<()> {
    let mut bytes_proofs: Vec<Vec<u8>> = Vec::new();

    for cons_proof in cons_proofs {
        let cons_proof: &String = cons_proof;

        bytes_proofs.push(
            cons_proof.from_base58().map_err(Context::new).to_result(
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
