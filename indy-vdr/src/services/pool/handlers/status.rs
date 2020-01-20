use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use failure::Context;

use futures::stream::StreamExt;

use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::{FromBase58, ToBase58};
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

use super::types::{LedgerStatus, Message};
use super::{check_cons_proofs, get_f, PoolRequest, RequestEvent, TimingResult};

#[derive(Debug)]
pub enum StatusRequestResult {
    CatchupTargetFound(
        Vec<u8>, // target_mt_root
        usize,   // target_mt_size
        Option<TimingResult>,
    ),
    CatchupTargetNotFound(LedgerError, Option<TimingResult>),
    Synced(Option<TimingResult>),
}

enum CatchupProgress {
    ConsensusNotReachable,
    ShouldBeStarted(
        Vec<u8>, //target_mt_root
        usize,   //target_mt_size
    ),
    NotNeeded,
    InProgress,
    Timeout,
}

pub async fn handle_status_request<Request: PoolRequest>(
    mut request: Request,
    merkle_tree: MerkleTree,
) -> LedgerResult<StatusRequestResult> {
    trace!("status request");
    let mut handler = StatusRequestHandler::new(merkle_tree, request.node_count());
    request.send_to_all(request.pool_config().reply_timeout)?;
    loop {
        let response = match request.next().await {
            Some(RequestEvent::Received(node_alias, _message, parsed)) => {
                let result = match parsed {
                    Message::LedgerStatus(ls) => handler.process_catchup_target(
                        ls.merkleRoot.clone(),
                        ls.txnSeqNo,
                        None,
                        &node_alias,
                        request.get_timing(),
                    ),
                    Message::ConsistencyProof(cp) => handler.process_catchup_target(
                        cp.newMerkleRoot.clone(),
                        cp.seqNoEnd,
                        Some(cp.hashes.clone()),
                        &node_alias,
                        request.get_timing(),
                    ),
                    _ => {
                        // FIXME - add req.unexpected(message) to raise an appropriate exception
                        return Err(err_msg(
                            LedgerErrorKind::InvalidState,
                            "Unexpected response",
                        ));
                    }
                };
                request.clean_timeout(node_alias)?;
                result
            }
            Some(RequestEvent::Timeout(ref node_alias)) => handler.process_catchup_target(
                "timeout".to_string(),
                0,
                None,
                node_alias,
                request.get_timing(),
            ),
            None => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        if response.is_some() {
            return Ok(response.unwrap());
        }
    }
}

#[derive(Debug)]
struct StatusRequestHandler {
    merkle_tree: MerkleTree,
    replies: HashMap<(String, usize, Option<Vec<String>>), HashSet<String>>,
    nodes_cnt: usize,
}

impl StatusRequestHandler {
    pub fn new(merkle_tree: MerkleTree, nodes_cnt: usize) -> Self {
        Self {
            merkle_tree,
            replies: HashMap::new(),
            nodes_cnt,
        }
    }

    pub fn process_catchup_target(
        &mut self,
        merkle_root: String,
        txn_seq_no: usize,
        cons_proof: Option<Vec<String>>,
        node_alias: &str,
        timing: Option<TimingResult>,
    ) -> Option<StatusRequestResult> {
        let key = (merkle_root, txn_seq_no, cons_proof);
        let contains = self
            .replies
            .get_mut(&key)
            .map(|set| {
                set.insert(node_alias.to_string());
            })
            .is_some();
        if !contains {
            self.replies
                .insert(key, HashSet::from_iter(vec![(node_alias.to_string())]));
        }

        match check_nodes_responses_on_status(
            &self.replies,
            &self.merkle_tree,
            self.nodes_cnt,
            get_f(self.nodes_cnt),
        ) {
            Ok(CatchupProgress::NotNeeded) => Some(StatusRequestResult::Synced(timing)),
            Ok(CatchupProgress::InProgress) => None,
            Ok(CatchupProgress::ConsensusNotReachable) => {
                //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
                Some(StatusRequestResult::CatchupTargetNotFound(
                    err_msg(LedgerErrorKind::PoolTimeout, "No consensus possible"),
                    timing,
                ))
            }
            Ok(CatchupProgress::ShouldBeStarted(target_mt_root, target_mt_size)) => Some(
                StatusRequestResult::CatchupTargetFound(target_mt_root, target_mt_size, timing),
            ),
            Ok(CatchupProgress::Timeout) => Some(StatusRequestResult::CatchupTargetNotFound(
                err_msg(LedgerErrorKind::PoolTimeout, "Sync timed out"),
                timing,
            )),
            Err(err) => Some(StatusRequestResult::CatchupTargetNotFound(err, timing)),
        }
    }
}

pub fn build_ledger_status_req(
    merkle: &MerkleTree,
    protocol_version: ProtocolVersion,
) -> LedgerResult<Message> {
    let lr = LedgerStatus {
        txnSeqNo: merkle.count(),
        merkleRoot: merkle.root_hash().as_slice().to_base58(),
        ledgerId: 0,
        ppSeqNo: None,
        viewNo: None,
        protocolVersion: Some(protocol_version as usize),
    };
    Ok(Message::LedgerStatus(lr))
}

fn check_nodes_responses_on_status(
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
            try_to_catch_up(most_popular_not_timeout_vote, merkle_tree)
        } else {
            if_consensus_reachable(nodes_votes, node_cnt, *votes_cnt, f)
        }
    } else if let Some((_, votes_cnt)) = timeout_votes {
        if *votes_cnt == node_cnt - f {
            Ok(CatchupProgress::Timeout)
        } else {
            if_consensus_reachable(nodes_votes, node_cnt, *votes_cnt, f)
        }
    } else {
        Ok(CatchupProgress::InProgress)
    }
}

fn if_consensus_reachable(
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

fn try_to_catch_up(
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
