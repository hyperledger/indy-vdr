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
    merkle: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<CatchupRequestResult> {
    trace!("catchup request");
    let mut handler = CatchupSingleHandler::new(merkle, target_mt_root, target_mt_size);
    let ack_timeout = request.pool_config().ack_timeout;
    request.send_to_any(1, ack_timeout)?;
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, _message, parsed)) => {
                match parsed {
                    Message::CatchupRep(cr) => {
                        match handler.process_catchup_reply(cr.load_txns()?, cr.consProof.clone()) {
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
            None => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        }
    }
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

#[derive(Debug)]
struct CatchupSingleHandler {
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
}

impl CatchupSingleHandler {
    fn new(merkle_tree: MerkleTree, target_mt_root: Vec<u8>, target_mt_size: usize) -> Self {
        Self {
            merkle_tree,
            target_mt_root,
            target_mt_size,
        }
    }

    fn process_catchup_reply(
        &mut self,
        txns: Vec<Vec<u8>>,
        cons_proof: Vec<String>,
    ) -> LedgerResult<Vec<Vec<u8>>> {
        let mut merkle = self.merkle_tree.clone();
        for txn in &txns {
            merkle.append(txn.clone())?;
        }
        check_cons_proofs(
            &merkle,
            &cons_proof,
            &self.target_mt_root,
            self.target_mt_size,
        )?;
        Ok(txns)
    }
}
