use futures::stream::StreamExt;

use crate::utils::error::prelude::*;

use super::types::Message;
use super::{NodeReplies, PoolRequest, ReplyState, RequestEvent, TimingResult};

pub type FullRequestResult = (NodeReplies<String>, Option<TimingResult>);

pub async fn handle_full_request<Request: PoolRequest>(
    mut request: Request,
    local_timeout: Option<i64>,
    nodes_to_send: Option<Vec<String>>,
) -> LedgerResult<FullRequestResult> {
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
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        if replies.len() == req_reply_count {
            return Ok((replies.result(), request.get_timing()));
        }
    }
}
