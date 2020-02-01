use std::hash::{Hash, Hasher};

use futures::stream::StreamExt;

use serde_json::Value as SJsonValue;

use ursa::bls::Generator;

use crate::common::error::prelude::*;
use crate::config::constants::DEFAULT_GENERATOR;
use crate::state_proof::{check_state_proof, result_without_state_proof};
use crate::utils::base58::FromBase58;

use super::types::Message;
use super::{
    min_consensus, ConsensusState, HashableValue, PoolRequest, ReplyState, RequestEvent,
    RequestResult, TimingResult,
};

pub async fn handle_single_request<Request: PoolRequest>(
    mut request: Request,
    state_proof_key: Option<Vec<u8>>,
    state_proof_timestamps: (Option<u64>, Option<u64>),
) -> LedgerResult<(RequestResult<String>, Option<TimingResult>)> {
    trace!("single request");
    let config = request.pool_config();
    let node_keys = request.node_keys();
    let total_nodes_count = request.node_count();
    let f = min_consensus(total_nodes_count);
    let mut replies = ReplyState::new();
    let mut state = ConsensusState::new();
    let generator: Generator =
        Generator::from_bytes(&DEFAULT_GENERATOR.from_base58()?).map_err(|err| {
            err_msg(
                LedgerErrorKind::Resource,
                format!("Error loading generator: {}", err.to_string()),
            )
        })?;

    request.send_to_any(config.request_read_nodes, config.ack_timeout)?;
    loop {
        let resend = match request.next().await {
            Some(RequestEvent::Received(node_alias, raw_msg, parsed)) => match parsed {
                Message::Reply(reply) => {
                    trace!("reply on single request");
                    if let Some(result) = reply.result() {
                        let result_without_proof = result_without_state_proof(result);
                        replies.add_reply(node_alias.clone(), true);
                        let hashable = HashableValue {
                            inner: result_without_proof,
                        };
                        let last_write_time = get_last_signed_time(result).unwrap_or(0);
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
                            return Ok((
                                RequestResult::Reply(if cnt > f { soonest } else { raw_msg }),
                                request.get_timing(),
                            ));
                        } else if state_proof_key.is_some() {
                            debug!("State proof verification failed for node: {}", node_alias);
                        }
                        request.clean_timeout(node_alias)?;
                        true
                    } else {
                        debug!("Error parsing result of reply from {}", node_alias);
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
                    debug!("Unexpected response from {} {:?}", node_alias, raw_msg);
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
                return Ok((
                    RequestResult::Failed(err_msg(
                        LedgerErrorKind::Network,
                        "Request ended prematurely",
                    )),
                    request.get_timing(),
                ))
            }
        };
        if replies.len() >= total_nodes_count {
            let err = {
                let counts = replies.counts();
                if counts.replies > 0 {
                    LedgerErrorKind::PoolNoConsensus.into()
                } else if counts.failed > 0 {
                    let failed = replies.sample_failed().unwrap();
                    LedgerErrorKind::PoolRequestFailed(failed).into()
                } else {
                    LedgerErrorKind::PoolTimeout.into()
                }
            };
            return Ok((RequestResult::Failed(err), request.get_timing()));
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

pub fn get_last_signed_time(reply_result: &SJsonValue) -> Option<u64> {
    let c = parse_reply_metadata(reply_result);
    c.ok().and_then(|resp| resp.last_txn_time)
}

pub fn parse_reply_metadata(reply_result: &SJsonValue) -> LedgerResult<ResponseMetadata> {
    let response_metadata = match reply_result["ver"].as_str() {
        None => parse_transaction_metadata_v0(reply_result),
        Some("1") => parse_transaction_metadata_v1(reply_result),
        ver => {
            return Err(input_err(format!(
                "Unsupported transaction response version: {:?}",
                ver
            )))
        }
    };

    trace!(
        "parse_response_metadata >> response_metadata: {:?}",
        response_metadata
    );

    Ok(response_metadata)
}

fn parse_transaction_metadata_v0(message: &SJsonValue) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["seqNo"].as_u64(),
        txn_time: message["txnTime"].as_u64(),
        last_txn_time: message["state_proof"]["multi_signature"]["value"]["timestamp"].as_u64(),
        last_seq_no: None,
    }
}

fn parse_transaction_metadata_v1(message: &SJsonValue) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["txnMetadata"]["seqNo"].as_u64(),
        txn_time: message["txnMetadata"]["txnTime"].as_u64(),
        last_txn_time: message["multiSignature"]["signedState"]["stateMetadata"]["timestamp"]
            .as_u64(),
        last_seq_no: None,
    }
}

#[derive(Debug)]
pub struct ResponseMetadata {
    pub seq_no: Option<u64>,
    pub txn_time: Option<u64>,
    pub last_txn_time: Option<u64>,
    pub last_seq_no: Option<u64>,
}
