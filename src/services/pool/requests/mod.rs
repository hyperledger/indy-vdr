use std::collections::HashMap;
use std::iter::FromIterator;
use std::time::{Duration, SystemTime};

use serde_json::{self, Value as SJsonValue};

use super::pool;
use super::state_proof;
use super::types::{self, Message, PoolConfig};

use crate::domain::ledger::constants;
use crate::utils::base58::FromBase58;
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

pub mod catchup;
pub mod single;
pub mod status;

new_handle_type!(RequestHandle, RQ_COUNTER);

pub const REQUESTS_FOR_STATE_PROOFS: [&str; 11] = [
    constants::GET_NYM,
    constants::GET_TXN_AUTHR_AGRMT,
    constants::GET_TXN_AUTHR_AGRMT_AML,
    constants::GET_SCHEMA,
    constants::GET_CRED_DEF,
    constants::GET_ATTR,
    constants::GET_REVOC_REG,
    constants::GET_REVOC_REG_DEF,
    constants::GET_REVOC_REG_DELTA,
    constants::GET_AUTH_RULE,
    constants::GET_TXN,
];

const REQUEST_FOR_FULL: [&str; 2] = [constants::POOL_RESTART, constants::GET_VALIDATOR_INFO];

pub const REQUESTS_FOR_STATE_PROOFS_IN_THE_PAST: [&str; 5] = [
    constants::GET_REVOC_REG,
    constants::GET_REVOC_REG_DELTA,
    constants::GET_TXN_AUTHR_AGRMT,
    constants::GET_TXN_AUTHR_AGRMT_AML,
    constants::GET_TXN,
];

pub const REQUESTS_FOR_MULTI_STATE_PROOFS: [&str; 1] = [constants::GET_REVOC_REG_DELTA];

#[derive(Debug)]
pub enum RequestEvent {
    Received(
        String,  // node alias
        String,  // message
        Message, // parsed
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug)]
pub enum RequestExtEvent {
    Init(),
    Sent(
        String,     // node alias
        SystemTime, // send time
        usize,      // send index
    ),
    Received(
        String,     // node alias
        String,     // message
        Message,    // parsed
        SystemTime, // received time
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestDispatchTarget {
    AllNodes,
    AnyNodes(usize),
    SelectNodes(Vec<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestState {
    NotStarted,
    Active,
    Terminated,
}

impl std::fmt::Display for RequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::NotStarted => "NotStarted",
            Self::Active => "Active",
            Self::Terminated => "Terminated",
        };
        f.write_str(state)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestTimeout {
    Default,
    Ack,
    #[allow(dead_code)]
    Seconds(i64),
}

impl RequestTimeout {
    pub fn expand(&self, config: &PoolConfig) -> i64 {
        match self {
            Self::Default => config.reply_timeout,
            Self::Ack => config.ack_timeout,
            Self::Seconds(n) => *n,
        }
    }
}

pub type TimingResult = HashMap<String, f32>;

#[derive(Debug)]
pub struct RequestTiming {
    replies: HashMap<String, (SystemTime, f32)>,
}

impl RequestTiming {
    pub fn new() -> Self {
        Self {
            replies: HashMap::new(),
        }
    }

    pub fn sent(&mut self, node_alias: &str, send_time: SystemTime) {
        self.replies
            .insert(node_alias.to_owned(), (send_time, -1.0));
    }

    pub fn received(&mut self, node_alias: &str, recv_time: SystemTime) {
        self.replies.get_mut(node_alias).map(|node| {
            let duration = recv_time
                .duration_since(node.0)
                .unwrap_or(Duration::new(0, 0))
                .as_secs_f32();
            node.1 = duration;
        });
    }

    pub fn get_result(&self) -> Option<TimingResult> {
        Some(HashMap::from_iter(
            self.replies.iter().map(|(k, (_, v))| (k.clone(), *v)),
        ))
    }
}

pub fn serialize_message(message: &types::Message) -> LedgerResult<(String, String)> {
    let req_id = message.request_id().unwrap_or("".to_owned());
    let req_json = serde_json::to_string(&message).map_err(|err| {
        err_msg(
            LedgerErrorKind::InvalidState,
            format!("Cannot serialize request: {:?}", err),
        )
    })?;
    Ok((req_id, req_json))
}

fn get_f(cnt: usize) -> usize {
    if cnt < 4 {
        return 0;
    }
    (cnt - 1) / 3
}

fn check_cons_proofs(
    mt: &MerkleTree,
    cons_proofs: &Vec<String>,
    target_mt_root: &Vec<u8>,
    target_mt_size: usize,
) -> LedgerResult<()> {
    let mut bytes_proofs: Vec<Vec<u8>> = Vec::new();

    for cons_proof in cons_proofs {
        let cons_proof: &String = cons_proof;

        bytes_proofs.push(
            cons_proof.from_base58().to_result(
                LedgerErrorKind::InvalidStructure,
                "Can't decode node consistency proof",
            )?, // FIXME: review kind
        );
    }

    if !mt.consistency_proof(target_mt_root, target_mt_size, &bytes_proofs)? {
        return Err(err_msg(
            LedgerErrorKind::InvalidState,
            "Consistency proof verification failed",
        )); // FIXME: review kind
    }

    Ok(())
}

fn get_msg_result_without_state_proof(msg: &str) -> LedgerResult<(SJsonValue, SJsonValue)> {
    let msg = serde_json::from_str::<SJsonValue>(msg).to_result(
        LedgerErrorKind::InvalidStructure,
        "Response is malformed json",
    )?;

    let msg_result = msg["result"].clone();

    let mut msg_result_without_proof: SJsonValue = msg_result.clone();
    msg_result_without_proof
        .as_object_mut()
        .map(|obj| obj.remove("state_proof"));

    if msg_result_without_proof["data"].is_object() {
        msg_result_without_proof["data"]
            .as_object_mut()
            .map(|obj| obj.remove("stateProofFrom"));
    }

    Ok((msg_result, msg_result_without_proof))
}
