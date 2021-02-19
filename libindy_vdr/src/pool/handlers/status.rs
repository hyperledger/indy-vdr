use futures_util::stream::StreamExt;

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::utils::base58;

use super::types::Message;
use super::{
    check_cons_proofs, min_consensus, ConsensusState, PoolRequest, ReplyState, RequestEvent,
    RequestResult, TimingResult,
};

pub type CatchupTarget = (Vec<u8>, usize);

pub async fn handle_status_request<R: PoolRequest>(
    request: &mut R,
    merkle_tree: MerkleTree,
) -> VdrResult<(RequestResult<Option<CatchupTarget>>, Option<TimingResult>)> {
    trace!("status request");
    let config = request.pool_config();
    let total_node_count = request.node_count();
    let mut replies = ReplyState::new();
    let mut consensus = ConsensusState::new();
    let f = min_consensus(total_node_count);
    request.send_to_all(config.reply_timeout)?;
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
            Some(RequestEvent::Timeout(node_alias)) => {
                replies.add_timeout(node_alias);
            }
            None => {
                return Ok((
                    RequestResult::Failed(err_msg(
                        VdrErrorKind::PoolTimeout,
                        "Request was interrupted",
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
            f,
        ) {
            Ok(CatchupProgress::NotNeeded) => {
                return Ok((RequestResult::Reply(None), request.get_timing()));
            }
            Ok(CatchupProgress::InProgress) => {}
            Ok(CatchupProgress::NoConsensus) => {
                return Ok((
                    RequestResult::Failed(replies.get_error()),
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
) -> VdrResult<CatchupProgress> {
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
) -> VdrResult<CatchupProgress> {
    let &(ref target_mt_root, target_mt_size, ref hashes) = ledger_status;
    let cur_mt_size = merkle_tree.count();
    let cur_mt_hash = base58::encode(merkle_tree.root_hash());

    if target_mt_size == cur_mt_size {
        if cur_mt_hash.eq(target_mt_root) {
            Ok(CatchupProgress::NotNeeded)
        } else {
            Err(input_err(
                "Ledger merkle tree is not acceptable for current tree.",
            ))
        }
    } else if target_mt_size > cur_mt_size {
        let target_mt_root = base58::decode(target_mt_root)
            .with_input_err("Can't parse target MerkleTree hash from nodes responses")?;

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
        Err(input_err("Local merkle tree greater than mt from ledger"))
    }
}
