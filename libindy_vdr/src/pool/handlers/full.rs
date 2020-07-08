use futures_util::stream::StreamExt;

use crate::common::error::prelude::*;

use super::types::{Message, NodeReplies, RequestResult, TimingResult};
use super::{PoolRequest, ReplyState, RequestEvent};

pub async fn handle_full_request<R: PoolRequest>(
    request: &mut R,
    nodes_to_send: Option<Vec<String>>,
    local_timeout: Option<i64>,
) -> VdrResult<(RequestResult<NodeReplies<String>>, Option<TimingResult>)> {
    trace!("full request");
    let timeout = local_timeout.unwrap_or(request.pool_config().reply_timeout);
    let req_reply_count = if let Some(nodes) = nodes_to_send {
        request.send_to(nodes, timeout)?.len()
    } else {
        request.send_to_all(timeout)?;
        request.node_count()
    };
    let mut replies = ReplyState::new();
    loop {
        match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => {
                match parsed {
                    Message::Reply(_) => {
                        trace!("reply on full request");
                        replies.add_reply(node_alias.clone(), raw_msg);
                    }
                    Message::ReqACK(_) => {
                        continue;
                    }
                    _ => {
                        trace!("error on full request");
                        replies.add_failed(node_alias.clone(), raw_msg); // ReqNACK, Reject
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
                        VdrErrorKind::PoolTimeout,
                        "Request was interrupted",
                    )),
                    request.get_timing(),
                ))
            }
        };
        if replies.len() == req_reply_count {
            return Ok((RequestResult::Reply(replies.result()), request.get_timing()));
        }
    }
}
