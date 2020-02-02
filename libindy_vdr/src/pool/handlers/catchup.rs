use futures::stream::StreamExt;

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;

use super::types::Message;
use super::{check_cons_proofs, PoolRequest, RequestEvent, RequestResult, TimingResult};

pub async fn handle_catchup_request<Request: PoolRequest>(
    mut request: Request,
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<(RequestResult<Vec<Vec<u8>>>, Option<TimingResult>)> {
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
                                return Ok((RequestResult::Reply(txns), request.get_timing()))
                            }
                            Err(_) => {
                                request.clean_timeout(node_alias)?;
                                request.send_to_any(1, ack_timeout)?;
                            }
                        }
                    }
                    _ => {
                        // FIXME could be more tolerant of ReqNACK etc
                        return Ok((
                            RequestResult::Failed(err_msg(
                                LedgerErrorKind::Connection,
                                "Unexpected response",
                            )),
                            request.get_timing(),
                        ));
                    }
                }
            }
            Some(RequestEvent::Timeout(_node_alias)) => {
                request.send_to_any(1, ack_timeout)?;
            }
            None => {
                return Ok((
                    RequestResult::Failed(LedgerErrorKind::PoolTimeout.into()),
                    request.get_timing(),
                ))
            }
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
