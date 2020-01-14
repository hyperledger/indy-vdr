use serde_json;
use serde_json::Value as SJsonValue;

use super::types::*;
use crate::domain::ledger::constants;
use crate::utils::error::prelude::*;

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
pub enum PoolEvent {
    Connect(
        CommandHandle,
        Vec<String>, // transactions
    ),
    Exit(),
}

/*
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
