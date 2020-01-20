use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::stream::StreamExt;

use serde_json::Value as SJsonValue;

use ursa::bls::Generator;

use crate::domain::ledger::response::Message as LedgerMessage;
use crate::utils::base58::FromBase58;
use crate::utils::error::prelude::*;

use super::state_proof;
use super::types::{Message, NodeKeys, DEFAULT_GENERATOR};
use super::{
    get_f, get_msg_result_without_state_proof, ConsensusResult, ConsensusState, HashableValue,
    PoolRequest, ReplyState, RequestEvent,
};

pub async fn handle_single_request<Request: PoolRequest>(
    mut request: Request,
    state_proof_key: Option<Vec<u8>>,
    state_proof_timestamps: (Option<u64>, Option<u64>),
) -> LedgerResult<ConsensusResult<String>> {
    trace!("single request");
    let config = request.pool_config();
    let node_keys = request.node_keys();
    let total_nodes_count = request.node_count();
    let f = get_f(total_nodes_count);
    let mut replies = ReplyState::new();
    let mut state = ConsensusState::new();
    let generator: Generator =
        Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap();

    request.send_to_any(config.request_read_nodes, config.ack_timeout)?;
    loop {
        let resend = match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => match parsed {
                Message::Reply(_) => {
                    trace!("reply on single request");
                    if let Ok((result, result_without_proof)) =
                        get_msg_result_without_state_proof(&raw_msg)
                    {
                        replies.add_reply(node_alias.clone(), true);
                        let hashable = HashableValue {
                            inner: result_without_proof,
                        };
                        let last_write_time = get_last_signed_time(&raw_msg).unwrap_or(0);
                        trace!("last write {}", last_write_time);
                        let (cnt, soonest) = {
                            let set = state.insert(
                                hashable,
                                NodeResponse {
                                    node_alias: node_alias.clone(),
                                    timestamp: last_write_time,
                                    raw_msg: raw_msg.clone(),
                                },
                            );
                            (
                                set.len(),
                                set.iter()
                                    .max_by_key(|resp| resp.timestamp)
                                    .map(|resp| &resp.raw_msg)
                                    .unwrap_or(&raw_msg)
                                    .clone(),
                            )
                        };
                        if cnt > f
                            || check_state_proof(
                                &result,
                                f,
                                &generator,
                                &node_keys,
                                &raw_msg,
                                state_proof_key.as_ref().map(Vec::as_slice),
                                state_proof_timestamps,
                                last_write_time,
                                config.freshness_threshold,
                            )
                        {
                            return Ok(ConsensusResult::Reply(
                                if cnt > f { soonest } else { raw_msg },
                                request.get_timing(),
                            ));
                        }
                        false
                    } else {
                        replies.add_failed(node_alias.clone(), raw_msg);
                        request.clean_timeout(node_alias)?;
                        true
                    }
                }
                Message::ReqACK(_) => {
                    request.extend_timeout(node_alias.clone(), config.reply_timeout)?;
                    continue;
                }
                Message::ReqNACK(_) | Message::Reject(_) => {
                    replies.add_failed(node_alias.clone(), raw_msg);
                    request.clean_timeout(node_alias)?;
                    true
                }
                _ => {
                    replies.add_failed(node_alias.clone(), raw_msg);
                    request.clean_timeout(node_alias)?;
                    true
                }
            },
            Some(RequestEvent::Timeout(node_alias)) => {
                replies.add_timeout(node_alias);
                true
            }
            None => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Request ended prematurely",
                ))
            }
        };
        if replies.len() >= total_nodes_count {
            return Ok(ConsensusResult::NoConsensus(request.get_timing()));
        }
        if resend {
            request.send_to_any(2, config.ack_timeout)?;
        }
    }
}

#[derive(Debug)]
struct NodeResponse {
    raw_msg: String,
    node_alias: String,
    timestamp: u64,
}

impl PartialEq for NodeResponse {
    fn eq(&self, other: &NodeResponse) -> bool {
        self.node_alias == other.node_alias
    }
}

impl Eq for NodeResponse {}

impl Hash for NodeResponse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_alias.hash(state);
    }
}

pub fn get_last_signed_time(raw_msg: &str) -> Option<u64> {
    let c = parse_response_metadata(raw_msg);
    c.ok().and_then(|resp| resp.last_txn_time)
}

pub fn parse_response_metadata(raw_msg: &str) -> LedgerResult<ResponseMetadata> {
    trace!("parse_response_metadata << raw_msg: {:?}", raw_msg);

    let message: LedgerMessage<SJsonValue> = serde_json::from_str(raw_msg).to_result(
        LedgerErrorKind::InvalidTransaction,
        "Cannot deserialize transaction response",
    )?;
    if let LedgerMessage::Reply(response_object) = message {
        let response_result = response_object.result();

        let response_metadata = match response_result["ver"].as_str() {
            None => parse_transaction_metadata_v0(&response_result),
            Some("1") => parse_transaction_metadata_v1(&response_result),
            ver => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidTransaction,
                    format!("Unsupported transaction response version: {:?}", ver),
                ))
            }
        };

        trace!(
            "indy::services::pool::parse_response_metadata >> response_metadata: {:?}",
            response_metadata
        );

        Ok(response_metadata)
    } else {
        Err(err_msg(
            LedgerErrorKind::InvalidTransaction,
            "Error parsing transaction response",
        ))
    }
}

fn parse_transaction_metadata_v0(message: &serde_json::Value) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["seqNo"].as_u64(),
        txn_time: message["txnTime"].as_u64(),
        last_txn_time: message["state_proof"]["multi_signature"]["value"]["timestamp"].as_u64(),
        last_seq_no: None,
    }
}

fn parse_transaction_metadata_v1(message: &serde_json::Value) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["txnMetadata"]["seqNo"].as_u64(),
        txn_time: message["txnMetadata"]["txnTime"].as_u64(),
        last_txn_time: message["multiSignature"]["signedState"]["stateMetadata"]["timestamp"]
            .as_u64(),
        last_seq_no: None,
    }
}

fn check_state_proof(
    msg_result: &SJsonValue,
    f: usize,
    gen: &Generator,
    bls_keys: &NodeKeys,
    raw_msg: &str,
    sp_key: Option<&[u8]>,
    requested_timestamps: (Option<u64>, Option<u64>),
    last_write_time: u64,
    threshold: u64,
) -> bool {
    debug!("process_reply: Try to verify proof and signature >>");

    let proof_checking_res =
        match state_proof::parse_generic_reply_for_proof_checking(&msg_result, raw_msg, sp_key) {
            Some(parsed_sps) => {
                debug!("process_reply: Proof and signature are present");
                state_proof::verify_parsed_sp(parsed_sps, bls_keys, f, gen)
            }
            None => false,
        };

    let res = proof_checking_res
        && check_freshness(msg_result, requested_timestamps, last_write_time, threshold);

    debug!(
        "process_reply: Try to verify proof and signature << {}",
        res
    );
    res
}

fn check_freshness(
    msg_result: &SJsonValue,
    requested_timestamps: (Option<u64>, Option<u64>),
    last_write_time: u64,
    threshold: u64,
) -> bool {
    debug!(
        "check_freshness: requested_timestamps: {:?} >>",
        requested_timestamps
    );

    let res = match requested_timestamps {
        (Some(from), Some(to)) => {
            let left_last_write_time = extract_left_last_write_time(msg_result).unwrap_or(0);
            trace!("Last last signed time: {}", left_last_write_time);
            trace!("Last right signed time: {}", last_write_time);

            let left_time_for_freshness_check = from;
            let right_time_for_freshness_check = to;

            trace!(
                "Left time for freshness check: {}",
                left_time_for_freshness_check
            );
            trace!(
                "Right time for freshness check: {}",
                right_time_for_freshness_check
            );

            left_time_for_freshness_check <= threshold + left_last_write_time
                && right_time_for_freshness_check <= threshold + last_write_time
        }
        (None, Some(to)) => {
            let time_for_freshness_check = to;

            trace!("Last signed time: {}", last_write_time);
            trace!("Time for freshness check: {}", time_for_freshness_check);

            time_for_freshness_check <= threshold + last_write_time
        }
        (Some(from), None) => {
            let left_last_write_time = extract_left_last_write_time(msg_result).unwrap_or(0);

            trace!("Last last signed time: {}", left_last_write_time);
            trace!("Last right signed time: {}", last_write_time);

            let left_time_for_freshness_check = from;
            let time_for_freshness_check = get_cur_time();

            trace!(
                "Left time for freshness check: {}",
                left_time_for_freshness_check
            );
            trace!("Time for freshness check: {}", time_for_freshness_check);

            left_time_for_freshness_check <= threshold + left_last_write_time
                && time_for_freshness_check <= threshold + last_write_time
        }
        (None, None) => {
            let time_for_freshness_check = get_cur_time();

            trace!("Last signed time: {}", last_write_time);
            trace!("Time for freshness check: {}", time_for_freshness_check);

            time_for_freshness_check <= threshold + last_write_time
        }
    };

    debug!("check_freshness << {:?} ", res);

    res
}

fn get_cur_time() -> u64 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");
    let res = since_epoch.as_secs();
    trace!("Current time: {}", res);
    res
}

// #[logfn(Trace)]
fn extract_left_last_write_time(msg_result: &SJsonValue) -> Option<u64> {
    match msg_result["type"].as_str() {
        Some(crate::domain::ledger::constants::GET_REVOC_REG_DELTA) => {
            msg_result["data"]["stateProofFrom"]["multi_signature"]["value"]["timestamp"].as_u64()
        }
        _ => None,
    }
}

#[derive(Debug)]
pub struct ResponseMetadata {
    pub seq_no: Option<u64>,
    pub txn_time: Option<u64>,
    pub last_txn_time: Option<u64>,
    pub last_seq_no: Option<u64>,
}
