use std::iter::FromIterator;
use std::string::ToString;

use serde_json;

use super::genesis::PoolTransactions;
use super::handlers::{
    build_pool_catchup_request, build_pool_status_request, handle_catchup_request,
    handle_consensus_request, handle_full_request, handle_status_request, CatchupTarget,
};
use super::pool::Pool;
use super::requests::{PreparedRequest, RequestMethod};
use super::types::{NodeReplies, RequestResult, TimingResult};

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::utils::base58;

/// Perform a pool ledger status request to see if catchup is required
pub async fn perform_pool_status_request<T: Pool>(
    pool: &T,
    merkle_tree: MerkleTree,
) -> VdrResult<(RequestResult<Option<CatchupTarget>>, Option<TimingResult>)> {
    let (mt_root, mt_size) = (merkle_tree.root_hash(), merkle_tree.count());
    let message = build_pool_status_request(mt_root, mt_size, pool.get_config().protocol_version)?;
    let req_json = message.serialize()?.to_string();
    let mut request = pool.create_request("".to_string(), req_json).await?;
    handle_status_request(&mut request, merkle_tree).await
}

/// Perform a pool ledger catchup request to fetch the latest verifier pool transactions
pub async fn perform_pool_catchup_request<T: Pool>(
    pool: &T,
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> VdrResult<(RequestResult<Vec<Vec<u8>>>, Option<TimingResult>)> {
    let message = build_pool_catchup_request(merkle_tree.count(), target_mt_size)?;
    let req_json = message.serialize()?.to_string();
    let mut request = pool.create_request("".to_string(), req_json).await?;
    handle_catchup_request(&mut request, merkle_tree, target_mt_root, target_mt_size).await
}

/// Perform a pool ledger status request followed by a catchup request if necessary
pub async fn perform_refresh<T: Pool>(
    pool: &T,
) -> VdrResult<(Option<Vec<String>>, Option<TimingResult>)> {
    let merkle_tree = pool.get_merkle_tree().clone();
    let (result, timing) = perform_pool_status_request(pool, merkle_tree.clone()).await?;
    trace!("Got status result: {:?}", &result);
    match result {
        RequestResult::Reply(target) => match target {
            Some((target_mt_root, target_mt_size)) => {
                debug!(
                    "Catchup target found {} {} {:?}",
                    base58::encode(&target_mt_root),
                    target_mt_size,
                    timing
                );
                let (txns, timing) =
                    perform_catchup(pool, merkle_tree, target_mt_root, target_mt_size).await?;
                Ok((Some(txns), timing))
            }
            _ => {
                info!("No catchup required {:?}", timing);
                Ok((None, timing))
            }
        },
        RequestResult::Failed(err) => {
            warn!("Catchup target not found {:?}", timing);
            Err(err)
        }
    }
}

pub(crate) async fn perform_catchup<T: Pool>(
    pool: &T,
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> VdrResult<(Vec<String>, Option<TimingResult>)> {
    let (catchup_result, timing) =
        perform_pool_catchup_request(pool, merkle_tree, target_mt_root.clone(), target_mt_size)
            .await?;
    match catchup_result {
        RequestResult::Reply(ref txns) => {
            info!("Catchup completed {:?}", timing);
            let new_txns = PoolTransactions::from_transactions(txns);
            let json_txns = new_txns.encode_json()?;
            let reload_txns = PoolTransactions::from_json_transactions(&json_txns)?;
            if new_txns != reload_txns {
                return Err(err_msg(
                    VdrErrorKind::Unexpected,
                    "Error validating rount-trip for pool transactions",
                ));
            }
            Ok((json_txns, timing))
        }
        RequestResult::Failed(err) => {
            trace!("Catchup failed {:?}", timing);
            Err(err)
        }
    }
}

/// Fetch a ledger transaction
pub async fn perform_get_txn<T: Pool>(
    pool: &T,
    ledger_type: i32,
    seq_no: i32,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    let builder = pool.get_request_builder();
    let prepared = builder.build_get_txn_request(None, ledger_type, seq_no)?;
    perform_ledger_request(pool, &prepared).await
}

/// Dispatch a request to a specific set of nodes and collect the results
pub async fn perform_ledger_action<T: Pool>(
    pool: &T,
    req_id: String,
    req_json: String,
    node_aliases: Option<Vec<String>>,
    timeout: Option<i64>,
) -> VdrResult<(RequestResult<NodeReplies<String>>, Option<TimingResult>)> {
    let mut request = pool.create_request(req_id, req_json).await?;
    handle_full_request(&mut request, node_aliases, timeout).await
}

/// Dispatch a prepared ledger request to the appropriate handler
pub async fn perform_ledger_request<T: Pool>(
    pool: &T,
    prepared: &PreparedRequest,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    let mut request = pool
        .create_request(prepared.req_id.clone(), prepared.req_json.to_string())
        .await?;

    let (sp_key, sp_timestamps, is_read_req, sp_parser) = match &prepared.method {
        RequestMethod::Full {
            node_aliases,
            timeout,
        } => {
            let (result, timing) =
                handle_full_request(&mut request, node_aliases.clone(), *timeout).await?;
            return Ok((result.map_result(format_full_reply)?, timing));
        }
        RequestMethod::BuiltinStateProof {
            sp_key,
            sp_timestamps,
        } => (Some(sp_key.clone()), *sp_timestamps, true, None),
        RequestMethod::CustomStateProof {
            sp_parser,
            sp_timestamps,
        } => (None, *sp_timestamps, true, Some(sp_parser)),
        RequestMethod::ReadConsensus => (None, (None, None), true, None),
        RequestMethod::Consensus => (None, (None, None), false, None),
    };

    handle_consensus_request(&mut request, sp_key, sp_timestamps, is_read_req, sp_parser).await
}

/// Format a collection of node replies in the expected response format
pub(crate) fn format_full_reply<T>(replies: NodeReplies<T>) -> VdrResult<String>
where
    T: ToString,
{
    serde_json::to_string(&serde_json::Map::from_iter(replies.into_iter().map(
        |(node_alias, reply)| (node_alias, serde_json::Value::from(reply.to_string())),
    )))
    .with_input_err("Error serializing response")
}
