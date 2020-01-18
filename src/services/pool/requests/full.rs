use futures::stream::StreamExt;

use crate::utils::error::prelude::*;

use super::pool::Pool;
use super::types::Message;
use super::{NodeReplies, ReplyState, RequestEvent, RequestTimeout, TimingResult};

pub type FullRequestResult = (NodeReplies<String>, Option<TimingResult>);

pub async fn perform_full_request<T: Pool>(
    pool: &T,
    req_id: &str,
    req_json: &str,
    local_timeout: Option<RequestTimeout>,
    nodes_to_send: Option<Vec<String>>,
) -> LedgerResult<FullRequestResult> {
    trace!("full request");
    let mut req = pool
        .create_request(req_id.to_owned(), req_json.to_owned())
        .await?;
    let timeout = local_timeout.unwrap_or(RequestTimeout::Default);
    let req_reply_count = if let Some(nodes) = nodes_to_send {
        req.send_to(nodes, timeout)?.len()
    } else {
        req.send_to_all(timeout)?;
        req.node_count()
    };
    let mut replies = ReplyState::new();
    loop {
        match req.next().await {
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
                req.clean_timeout(node_alias)?;
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
            return Ok((replies.result(), req.get_timing()));
        }
    }
}
