use futures::stream::StreamExt;

use crate::utils::error::prelude::*;

use super::pool::Pool;
use super::types::Message;
use super::{
    get_f, get_msg_result_without_state_proof, ConsensusResult, ConsensusState, HashableValue,
    ReplyState, RequestEvent, RequestTimeout,
};

pub async fn perform_consensus_request(
    pool: &Pool,
    req_id: &str,
    req_json: &str,
) -> LedgerResult<ConsensusResult<String>> {
    trace!("consensus request");
    let mut req = pool
        .create_request(req_id.to_owned(), req_json.to_owned())
        .await?;
    let nodes = pool.nodes();
    let total_nodes_count = nodes.len();
    let f = get_f(total_nodes_count);
    let mut replies = ReplyState::new();
    let mut state = ConsensusState::new();

    req.send_to_all(RequestTimeout::Ack)?;
    loop {
        match req.next().await {
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
                                return Ok(ConsensusResult::Reply(raw_msg, req.get_timing()));
                            }
                        } else {
                            replies.add_failed(node_alias.clone(), raw_msg);
                        }
                        req.clean_timeout(node_alias.clone())?;
                    }
                    Message::ReqACK(_) => {
                        req.extend_timeout(node_alias.clone(), RequestTimeout::Default)?;
                        continue;
                    }
                    Message::ReqNACK(_) | Message::Reject(_) => {
                        replies.add_failed(node_alias.clone(), raw_msg);
                        req.clean_timeout(node_alias.clone())?;
                    }
                    _ => {
                        replies.add_failed(node_alias.clone(), raw_msg);
                        req.clean_timeout(node_alias.clone())?;
                    }
                };
            }
            Some(RequestEvent::Timeout(node_alias)) => {
                replies.add_timeout(node_alias.clone());
                req.clean_timeout(node_alias.clone())?;
            }
            None => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        if state.max_consensus() + total_nodes_count - replies.len() <= f {
            return Ok(ConsensusResult::NoConsensus(req.get_timing()));
        }
    }
}
