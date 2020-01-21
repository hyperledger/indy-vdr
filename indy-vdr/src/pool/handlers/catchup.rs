use futures::stream::StreamExt;

use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

use super::types::{CatchupReq, Message};
use super::{check_cons_proofs, PoolRequest, RequestEvent, TimingResult};

#[derive(Debug)]
pub enum CatchupRequestResult {
    Synced(
        Vec<Vec<u8>>, // new transactions
        Option<TimingResult>,
    ),
    Timeout(),
}

pub async fn handle_catchup_request<Request: PoolRequest>(
    mut request: Request,
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<CatchupRequestResult> {
    trace!("catchup request");
    let ack_timeout = request.pool_config().ack_timeout;
    request.send_to_any(1, ack_timeout)?;
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, _message, parsed)) => {
                match parsed {
                    Message::CatchupRep(cr) => {
                        match process_catchup_reply(
                            &merkle_tree,
                            &target_mt_root,
                            target_mt_size,
                            cr.load_txns()?,
                            cr.consProof.clone(),
                        ) {
                            Ok(txns) => {
                                return Ok(CatchupRequestResult::Synced(txns, request.get_timing()))
                            }
                            Err(_) => {
                                request.clean_timeout(node_alias)?;
                                request.send_to_any(1, ack_timeout)?;
                            }
                        }
                    }
                    _ => {
                        // FIXME - add req.unexpected(message) to raise an appropriate exception
                        // should be more tolerant of ReqNACK etc
                        return Err(err_msg(
                            LedgerErrorKind::InvalidState,
                            "Unexpected response",
                        ));
                    }
                }
            }
            Some(RequestEvent::Timeout(_node_alias)) => {
                request.send_to_any(1, ack_timeout)?;
            }
            None => return Ok(CatchupRequestResult::Timeout()),
        }
    }
}

fn process_catchup_reply(
    source_tree: &MerkleTree,
    target_mt_root: &Vec<u8>,
    target_mt_size: usize,
    txns: Vec<Vec<u8>>,
    cons_proof: Vec<String>,
) -> LedgerResult<Vec<Vec<u8>>> {
    let mut merkle = source_tree.clone();
    for txn in &txns {
        merkle.append(txn.clone())?;
    }
    check_cons_proofs(&merkle, &cons_proof, target_mt_root, target_mt_size)?;
    Ok(txns)
}

pub fn build_catchup_req(merkle: &MerkleTree, target_mt_size: usize) -> LedgerResult<Message> {
    if merkle.count() >= target_mt_size {
        return Err(err_msg(
            LedgerErrorKind::InvalidState,
            "No transactions to catch up",
        ));
    }
    let seq_no_start = merkle.count() + 1;
    let seq_no_end = target_mt_size;

    let cr = CatchupReq {
        ledgerId: 0,
        seqNoStart: seq_no_start,
        seqNoEnd: seq_no_end,
        catchupTill: target_mt_size,
    };
    Ok(Message::CatchupReq(cr))
}
