use failure::Context;

use futures::stream::StreamExt;

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::pool::ProtocolVersion;
use crate::utils::base58::{FromBase58, ToBase58};

use super::types::{LedgerStatus, Message};
use super::{
    check_cons_proofs, min_consensus, ConsensusState, PoolRequest, ReplyState, RequestEvent,
    RequestResult, TimingResult,
};

type CatchupTarget = (Vec<u8>, usize);

pub async fn handle_status_request<Request: PoolRequest>(
    mut request: Request,
    merkle_tree: MerkleTree,
) -> LedgerResult<(RequestResult<Option<CatchupTarget>>, Option<TimingResult>)> {
    trace!("status request");
    let total_node_count = request.node_count();
    let mut replies = ReplyState::new();
    let mut consensus = ConsensusState::new();
    request.send_to_all(request.pool_config().reply_timeout)?;
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => {
                match parsed {
                    Message::LedgerStatus(ls) => {
                        trace!("Received ledger status from {}", &node_alias);
                        replies.add_reply(node_alias.clone(), true);
                        let key = (ls.merkleRoot.clone(), ls.txnSeqNo, None);
                        consensus.insert(key, node_alias.clone());
                    }
                    Message::ConsistencyProof(cp) => {
                        trace!("Received consistency proof from {}", &node_alias);
                        replies.add_reply(node_alias.clone(), true);
                        let key = (cp.newMerkleRoot.clone(), cp.seqNoEnd, Some(cp.hashes));
                        consensus.insert(key, node_alias.clone());
                    }
                    Message::ReqACK(_) => continue,
                    Message::ReqNACK(_) | Message::Reject(_) => {
                        debug!("Status request failed for {}", &node_alias);
                        replies.add_failed(node_alias.clone(), raw_msg);
                    }
                    _ => {
                        debug!("Unexpected reply from {}", &node_alias);
                        replies.add_failed(node_alias.clone(), raw_msg);
                    }
                };
                request.clean_timeout(node_alias)?;
            }
            Some(RequestEvent::Timeout(ref node_alias)) => {
                replies.add_timeout(node_alias.clone());
            }
            None => {
                return Ok((
                    RequestResult::Failed(err_msg(
                        LedgerErrorKind::InvalidState,
                        "Request ended prematurely",
                    )),
                    request.get_timing(),
                ))
            }
        };
        match check_nodes_responses_on_status(
            &merkle_tree,
            &replies,
            &consensus,
            total_node_count,
            min_consensus(total_node_count),
        ) {
            Ok(CatchupProgress::NotNeeded) => {
                return Ok((RequestResult::Reply(None), request.get_timing()));
            }
            Ok(CatchupProgress::InProgress) => {}
            Ok(CatchupProgress::NoConsensus) => {
                return Ok((
                    RequestResult::Failed(err_msg(LedgerErrorKind::NoConsensus, "No consensus")),
                    request.get_timing(),
                ));
            }
            Ok(CatchupProgress::ShouldBeStarted(target_mt_root, target_mt_size)) => {
                return Ok((
                    RequestResult::Reply(Some((target_mt_root, target_mt_size))),
                    request.get_timing(),
                ));
            }
            Err(err) => return Ok((RequestResult::Failed(err), request.get_timing())),
        };
    }
}

enum CatchupProgress {
    ShouldBeStarted(
        Vec<u8>, //target_mt_root
        usize,   //target_mt_size
    ),
    NoConsensus,
    NotNeeded,
    InProgress,
}

fn check_nodes_responses_on_status<R>(
    merkle_tree: &MerkleTree,
    replies: &ReplyState<R>,
    consensus: &ConsensusState<(String, usize, Option<Vec<String>>), String>,
    total_nodes_count: usize,
    f: usize,
) -> LedgerResult<CatchupProgress> {
    let max_consensus = if let Some((most_popular_vote, votes_count)) = consensus.max_entry() {
        if votes_count > f {
            return try_to_catch_up(most_popular_vote, merkle_tree);
        }
        votes_count
    } else {
        0
    };
    if max_consensus + total_nodes_count - replies.len() <= f {
        return Ok(CatchupProgress::NoConsensus);
    }
    Ok(CatchupProgress::InProgress)
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
