use std::collections::HashMap;

use futures::stream::StreamExt;

use crate::utils::error::prelude::*;

use super::pool::Pool;
use super::types::Message;
use super::{RequestEvent, RequestTimeout, TimingResult};

pub type FullRequestResult = (HashMap<String, FullRequestReply>, Option<TimingResult>);

#[derive(Debug)]
pub enum FullRequestReply {
    Reply(String),
    Failed(String),
    Timeout(),
}

impl FullRequestReply {
    fn to_string(self) -> String {
        match self {
            Self::Reply(msg) => msg,
            Self::Failed(msg) => msg,
            Self::Timeout() => "timeout".to_owned(),
        }
    }
}

pub async fn perform_full_request(
    pool: &Pool,
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
        // FIXME could validate that nodes are in pool.nodes()
        let count = nodes.len();
        req.send_to(nodes, timeout)?;
        count
    } else {
        req.send_to_all(timeout)?;
        pool.nodes().len()
    };
    let mut replies = HashMap::new();
    loop {
        let (node_alias, reply) = match req.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => {
                match parsed {
                    Message::Reply(_) => {
                        trace!("reply on full request");
                        (node_alias, FullRequestReply::Reply(raw_msg))
                    }
                    Message::ReqACK(_) => {
                        continue;
                    }
                    _ => {
                        trace!("error on full request");
                        (node_alias, FullRequestReply::Failed(raw_msg)) // ReqNACK, Reject
                    }
                }
            }
            Some(RequestEvent::Timeout(node_alias)) => (node_alias, FullRequestReply::Timeout()),
            None => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        replies.insert(node_alias, reply);
        if replies.len() == req_reply_count {
            return Ok((replies, req.get_timing()));
        }
    }
}
