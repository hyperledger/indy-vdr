use futures::stream::StreamExt;

use crate::utils::error::prelude::*;

use super::types::Message;
use super::{
    get_msg_result_without_state_proof, min_consensus, ConsensusResult, ConsensusState,
    HashableValue, PoolRequest, ReplyState, RequestEvent,
};

pub async fn handle_consensus_request<Request: PoolRequest>(
    mut request: Request,
) -> LedgerResult<ConsensusResult<String>> {
    trace!("consensus request");
    let total_nodes_count = request.node_count();
    let f = min_consensus(total_nodes_count);
    let mut replies = ReplyState::new();
    let mut state = ConsensusState::new();
    let config = request.pool_config();

    request.send_to_all(config.ack_timeout)?;
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => {
                match parsed {
                    Message::Reply(_) => {
                        trace!("reply on consensus request");
                        if let Ok((_result, result_without_proof)) =
                            get_msg_result_without_state_proof(&raw_msg)
                        {
                            replies.add_reply(node_alias.clone(), true);
                            let hashable = HashableValue {
                                inner: result_without_proof,
                            };
                            let cnt = state.insert(hashable, node_alias.clone()).len();
                            if cnt > f {
                                return Ok(ConsensusResult::Reply(raw_msg, request.get_timing()));
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
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        if state.max_consensus() + total_nodes_count - replies.len() <= f {
            return Ok(ConsensusResult::NoConsensus(request.get_timing()));
        }
    }
}
