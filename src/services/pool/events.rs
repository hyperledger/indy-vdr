use std::collections::HashMap;

use serde_json;
use serde_json::Value as SJsonValue;

use super::types::*;
use crate::domain::ledger::constants;
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

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

pub const COMMAND_EXIT: &str = "exit";
pub const COMMAND_CONNECT: &str = "connect";
pub const COMMAND_REFRESH: &str = "refresh";

#[derive(Debug)]
pub enum PoolEvent {
    /*
    Open(CommandHandle, Option<JsonTransactions>),
    Refresh(CommandHandle, Option<JsonTransactions>),
    NodeReply(
        String, // reply
        String, // node alias
    ),
    Close(CommandHandle),
    #[allow(dead_code)] //FIXME
    PoolOutdated,
    SendRequest(
        CommandHandle,
        String,         // request
        Option<i32>,    // timeout
        Option<String>, // node list
    ),
    Timeout(
        String, //req_id
        String, //node alias
    ),
    NetworkerDone
    */
    Connect(
        CommandHandle,
        Vec<String>, // transactions
    ),
    Exit(),
    SubmitAck(
        String, // request ID
        LedgerResult<()>,
    ),
    Response(
        String, // request ID
        LedgerResult<String>,
    ),
    CatchupTargetFound(
        String,                       // request ID
        Vec<u8>,                      // target_mt_root
        usize,                        // target_mt_size
        Option<HashMap<String, f32>>, // node: duration
    ),
    CatchupTargetNotFound(
        String, // request ID
        LedgerError,
        Option<HashMap<String, f32>>, // node: duration
    ),
    StatusSynced(
        String,                       // request ID
        Option<HashMap<String, f32>>, // node: duration
    ),
    Synced(
        String, // request ID
        Option<MerkleTree>,
    ),
    // NodesBlacklisted,
}

/*
#[derive(Debug)]
pub enum PoolUpdate {
    OpenAck(CommandHandle, PoolHandle, LedgerResult<()>),
    CloseAck(CommandHandle, PoolHandle, LedgerResult<()>),
    RefreshAck(CommandHandle, PoolHandle, LedgerResult<()>),
    SubmitAck(CommandHandle, PoolHandle, LedgerResult<String>),
}
*/

#[derive(Clone, Debug)]
pub enum RequestEvent {
    StatusReq(MerkleTree),
    StatusRep(
        LedgerStatus,
        String, //node alias
    ),
    CatchupReq(
        MerkleTree,
        usize,   // target mt size
        Vec<u8>, // target mt root
    ),
    Timeout(
        String, //req_id
        String, //node_alias
    ),
    CatchupRep(
        CatchupRep,
        String, // node_alias
    ),
    CustomSingleRequest(
        String,                     // message
        String,                     // req_id
        Option<Vec<u8>>,            // expected key for State Proof in Reply,
        (Option<u64>, Option<u64>), // expected timestamps for freshness comparison
    ),
    CustomConsensusRequest(
        String, // message
        String, // req_id
    ),
    CustomFullRequest(
        String,         // message
        String,         // req_id
        Option<i32>,    // timeout
        Option<String>, // nodes
    ),
    ConsistencyProof(
        ConsistencyProof,
        String, //node alias
    ),
    Reply(
        Reply,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    ReqACK(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    ReqNACK(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    Reject(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    PoolLedgerTxns,
    Ping,
    Pong,
    Terminate,
}

/*
#[derive(Debug)]
pub enum RequestUpdate {
    SubmitAck(Vec<CommandHandle>, LedgerResult<String>),
    Synced(MerkleTree),
    CatchupTargetFound(
        Vec<u8>, //target_mt_root
        usize,   //target_mt_size
        MerkleTree,
    ),
    CatchupRestart(MerkleTree),
    CatchupTargetNotFound(LedgerError),
    NodesBlacklisted,
}
*/

impl RequestEvent {
    pub fn get_req_id(&self) -> String {
        match *self {
            RequestEvent::CustomSingleRequest(_, ref id, _, _) => id.to_string(),
            RequestEvent::CustomConsensusRequest(_, ref id) => id.to_string(),
            RequestEvent::CustomFullRequest(_, ref id, _, _) => id.to_string(),
            RequestEvent::Reply(_, _, _, ref id) => id.to_string(),
            RequestEvent::ReqACK(_, _, _, ref id) => id.to_string(),
            RequestEvent::ReqNACK(_, _, _, ref id) => id.to_string(),
            RequestEvent::Reject(_, _, _, ref id) => id.to_string(),
            _ => "".to_string(),
        }
    }

    /*
    pub fn from_pool_event(
        event: PoolEvent,
        protocol_version: ProtocolVersion, // FIXME: pass in state proof parser instead of version
    ) -> Option<RequestEvent> {
        match event {
            PoolEvent::NodeReply(msg, node_alias) => {
                _parse_msg(&msg).and_then(|parsed| match parsed {
                    Message::CatchupReq(_) => {
                        warn!("ignoring catchup request");
                        None
                        // RequestEvent::CatchupReq(MerkleTree::default(), 0, vec![])
                    }
                    Message::CatchupRep(rep) => Some(RequestEvent::CatchupRep(rep, node_alias)),
                    Message::LedgerStatus(ls) => Some(RequestEvent::StatusRep(ls, node_alias)),
                    Message::ConsistencyProof(cp) => {
                        Some(RequestEvent::ConsistencyProof(cp, node_alias))
                    }
                    Message::Reply(rep) => {
                        let req_id = rep.req_id();
                        Some(RequestEvent::Reply(
                            rep,
                            msg,
                            node_alias,
                            req_id.to_string(),
                        ))
                    }
                    Message::ReqACK(rep) => {
                        let req_id = rep.req_id();
                        Some(RequestEvent::ReqACK(
                            rep,
                            msg,
                            node_alias,
                            req_id.to_string(),
                        ))
                    }
                    Message::ReqNACK(rep) => {
                        let req_id = rep.req_id();
                        Some(RequestEvent::ReqNACK(
                            rep,
                            msg,
                            node_alias,
                            req_id.to_string(),
                        ))
                    }
                    Message::Reject(rep) => {
                        let req_id = rep.req_id();
                        Some(RequestEvent::Reject(
                            rep,
                            msg,
                            node_alias,
                            req_id.to_string(),
                        ))
                    }
                    Message::PoolLedgerTxns(_) => Some(RequestEvent::PoolLedgerTxns),
                    Message::Ping => Some(RequestEvent::Ping),
                    Message::Pong => Some(RequestEvent::Pong),
                })
            }
            PoolEvent::SendRequest(_, msg, timeout, nodes) => {
                let parsed_req = _parse_req_id_and_op(&msg);
                if let Ok((ref req, ref req_id, ref op)) = parsed_req {
                    if REQUEST_FOR_FULL.contains(&op.as_str()) {
                        Some(RequestEvent::CustomFullRequest(
                            msg,
                            req_id.clone(),
                            timeout,
                            nodes,
                        ))
                    } else if timeout.is_some() || nodes.is_some() {
                        error!("Timeout {:?} or nodes {:?} is specified for non-supported request operation type {}",
                               timeout, nodes, op);
                        None
                    } else if REQUESTS_FOR_STATE_PROOFS.contains(&op.as_str()) {
                        let key = super::state_proof::parse_key_from_request_for_builtin_sp(
                            &req,
                            protocol_version,
                        );
                        let timestamps = _parse_timestamp_from_req_for_builtin_sp(req, &op);
                        Some(RequestEvent::CustomSingleRequest(
                            msg,
                            req_id.clone(),
                            key,
                            timestamps,
                        ))
                    }
                    /*
                    FIXME custom state proof parser
                    else if PoolService::get_sp_parser(&op.as_str()).is_some() {
                        Some(RequestEvent::CustomSingleRequest(
                            msg,
                            req_id.clone(),
                            None,
                            (None, None),
                        ))
                    }*/
                    else {
                        Some(RequestEvent::CustomConsensusRequest(msg, req_id.clone()))
                    }
                } else {
                    error!("Can't parse parsed_req or op from message {}", msg);
                    None
                }
            }
            PoolEvent::Timeout(req_id, node_alias) => {
                Some(RequestEvent::Timeout(req_id, node_alias))
            }
            _ => None,
        }
    }
    */
}

/*
pub trait UpdateHandler {
    fn send(&mut self, update: PoolUpdate) -> LedgerResult<()>;
}

pub struct MockUpdateHandler {
    pub events: Vec<PoolUpdate>,
}

impl MockUpdateHandler {
    pub fn new() -> Self {
        Self { events: vec![] }
    }
}

impl UpdateHandler for MockUpdateHandler {
    fn send(&mut self, update: PoolUpdate) -> LedgerResult<()> {
        self.events.push(update);
        Ok(())
    }
}
*/

fn _parse_timestamp_from_req_for_builtin_sp(
    req: &SJsonValue,
    op: &str,
) -> (Option<u64>, Option<u64>) {
    if !REQUESTS_FOR_STATE_PROOFS_IN_THE_PAST.contains(&op) {
        return (None, None);
    }

    if op == constants::GET_TXN {
        return (None, Some(0));
    }

    match op {
        constants::GET_REVOC_REG
        | constants::GET_TXN_AUTHR_AGRMT
        | constants::GET_TXN_AUTHR_AGRMT_AML => (None, req["operation"]["timestamp"].as_u64()),
        constants::GET_REVOC_REG_DELTA => (
            req["operation"]["from"].as_u64(),
            req["operation"]["to"].as_u64(),
        ),
        _ => (None, None),
    }
}

fn _parse_msg(msg: &str) -> Option<Message> {
    Message::from_raw_str(msg).map_err(map_err_trace!()).ok()
}

fn _parse_req_id_and_op(msg: &str) -> LedgerResult<(SJsonValue, String, String)> {
    let req_json = _get_req_json(msg)?;

    let req_id = req_json["reqId"]
        .as_u64()
        .ok_or_else(|| err_msg(LedgerErrorKind::InvalidStructure, "No reqId in request"))?
        .to_string();

    let op = req_json["operation"]["type"]
        .as_str()
        .ok_or_else(|| {
            err_msg(
                LedgerErrorKind::InvalidStructure,
                "No operation type in request",
            )
        })?
        .to_string();

    Ok((req_json, req_id, op))
}

fn _get_req_json(msg: &str) -> LedgerResult<SJsonValue> {
    serde_json::from_str(msg).to_result(LedgerErrorKind::InvalidStructure, "Invalid request json")
    // FIXME: Review kind
}
