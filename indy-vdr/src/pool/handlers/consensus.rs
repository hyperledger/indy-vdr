use futures::stream::StreamExt;

use crate::common::error::prelude::*;
use crate::state_proof::result_without_state_proof;

use super::types::Message;
use super::{
    min_consensus, ConsensusState, HashableValue, PoolRequest, ReplyState, RequestEvent,
    RequestResult, TimingResult,
};

pub async fn handle_consensus_request<Request: PoolRequest>(
    mut request: Request,
) -> LedgerResult<(RequestResult<String>, Option<TimingResult>)> {
    trace!("consensus request");
    let total_nodes_count = request.node_count();
    let f = min_consensus(total_nodes_count);
    let mut replies = ReplyState::new();
    let mut consensus = ConsensusState::new();
    let config = request.pool_config();

    request.send_to_all(config.ack_timeout)?;
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => {
                match parsed {
                    Message::Reply(reply) => {
                        trace!("reply on consensus request");
                        if let Some(result) = reply.result() {
                            let result_without_proof = result_without_state_proof(result);
                            replies.add_reply(node_alias.clone(), true);
                            let hashable = HashableValue {
                                inner: result_without_proof,
                            };
                            let cnt = consensus.insert(hashable, node_alias.clone()).len();
                            if cnt > f {
                                return Ok((RequestResult::Reply(raw_msg), request.get_timing()));
                            }
                        } else {
                            replies.add_failed(node_alias.clone(), raw_msg);
                        }
                    }
                    Message::ReqACK(_) => {
                        request.extend_timeout(node_alias.clone(), config.reply_timeout)?;
                        continue;
                    }
                    Message::ReqNACK(_) | Message::Reject(_) => {
                        replies.add_failed(node_alias.clone(), raw_msg);
                    }
                    _ => {
                        replies.add_failed(node_alias.clone(), raw_msg);
                    }
                }
                request.clean_timeout(node_alias)?;
            }
            Some(RequestEvent::Timeout(node_alias)) => {
                replies.add_timeout(node_alias);
            }
            None => {
                return Ok((
                    RequestResult::Failed(err_msg(
                        LedgerErrorKind::Network,
                        "Request ended prematurely",
                    )),
                    request.get_timing(),
                ))
            }
        };
        if consensus.max_len() + total_nodes_count - replies.len() <= f {
            return Ok((
                RequestResult::Failed(LedgerErrorKind::NoConsensus.into()),
                request.get_timing(),
            ));
        }
    }
}
