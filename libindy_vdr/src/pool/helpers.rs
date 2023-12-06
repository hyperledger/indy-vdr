use std::iter::FromIterator;
use std::string::ToString;

use serde_json;

use super::genesis::PoolTransactions;
use super::handlers::{
    build_pool_catchup_request, build_pool_status_request, handle_catchup_request,
    handle_consensus_request, handle_full_request, handle_status_request, CatchupTarget,
};
use super::manager::Pool;
use super::requests::{PoolRequest, PreparedRequest, RequestMethod};
use super::types::{NodeReplies, RequestResult, RequestResultMeta};

use crate::common::error::prelude::*;
use crate::pool::LedgerType;
use crate::utils::base58;

/// Perform a pool ledger status request to see if catchup is required
pub async fn perform_pool_status_request<T: Pool>(
    pool: &T,
) -> VdrResult<(RequestResult<Option<CatchupTarget>>, RequestResultMeta)> {
    let (mt_root, mt_size) = pool.get_merkle_tree_info();

    if pool.get_refreshed() {
        trace!("Performing fast status check");
        match perform_get_txn(pool, LedgerType::POOL.to_id(), 1).await {
            Ok((RequestResult::Reply(reply), res_meta)) => {
                if let Ok(body) = serde_json::from_str::<serde_json::Value>(&reply) {
                    if let (Some(status_root_hash), Some(status_txn_count)) = (
                        body["result"]["data"]["rootHash"].as_str(),
                        body["result"]["data"]["ledgerSize"].as_u64(),
                    ) {
                        let target = if status_root_hash == mt_root
                            && status_txn_count == mt_size as u64
                        {
                            debug!("Fast status check succeeded, pool state is up to date");
                            None
                        } else {
                            debug!("Fast status check got catchup target: {}", status_root_hash);
                            let target_mt_hash = base58::decode(status_root_hash)
                                .map_err(|_| invalid!("Can't decode status target root hash"))?;
                            Some((
                                target_mt_hash,
                                status_txn_count as usize,
                                res_meta.state_proof.keys().cloned().collect(),
                            ))
                        };
                        return Ok((RequestResult::Reply(target), res_meta));
                    } else {
                        warn!("Error retrieving transaction count from fast status request");
                    }
                } else {
                    warn!("Error parsing response from fast status request");
                }
            }
            Ok((RequestResult::Failed(err), _)) | Err(err) => {
                warn!("Failed fast status request: {err}")
            }
        }
    }

    let message = build_pool_status_request(mt_root, mt_size, pool.get_config().protocol_version)?;
    let req_json = message.serialize()?.to_string();
    let mut request = pool.create_request("".to_string(), req_json).await?;
    handle_status_request(&mut request, pool.get_merkle_tree()).await
}

/// Perform a pool ledger catchup request to fetch the latest verifier pool transactions
pub async fn perform_pool_catchup_request<T: Pool>(
    pool: &T,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
    preferred_nodes: Option<Vec<String>>,
) -> VdrResult<(RequestResult<Vec<Vec<u8>>>, RequestResultMeta)> {
    let message = build_pool_catchup_request(pool.get_merkle_tree().count(), target_mt_size)?;
    let req_json = message.serialize()?.to_string();
    let mut request = pool.create_request("".to_string(), req_json).await?;
    if let Some(nodes) = preferred_nodes {
        request.set_preferred_nodes(&nodes);
    }
    handle_catchup_request(
        &mut request,
        pool.get_merkle_tree(),
        target_mt_root,
        target_mt_size,
    )
    .await
}

/// Perform a pool ledger status request followed by a catchup request if necessary
pub async fn perform_refresh<T: Pool>(
    pool: &T,
) -> VdrResult<(Option<PoolTransactions>, RequestResultMeta)> {
    let (result, meta) = perform_pool_status_request(pool).await?;
    trace!("Got status result: {:?}", &result);
    match result {
        RequestResult::Reply(target) => match target {
            Some((target_mt_root, target_mt_size, nodes)) => {
                debug!(
                    "Catchup target found {} {} {:?}",
                    base58::encode(&target_mt_root),
                    target_mt_size,
                    meta
                );
                let (txns, meta) =
                    perform_catchup(pool, target_mt_root, target_mt_size, Some(nodes)).await?;
                Ok((Some(txns), meta))
            }
            _ => {
                info!("No catchup required {:?}", meta);
                Ok((None, meta))
            }
        },
        RequestResult::Failed(err) => {
            warn!("Catchup target not found {:?}", meta);
            Err(err)
        }
    }
}

pub(crate) async fn perform_catchup<T: Pool>(
    pool: &T,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
    preferred_nodes: Option<Vec<String>>,
) -> VdrResult<(PoolTransactions, RequestResultMeta)> {
    let mut new_txns = pool.get_transactions();
    let (catchup_result, meta) = perform_pool_catchup_request(
        pool,
        target_mt_root.clone(),
        target_mt_size,
        preferred_nodes,
    )
    .await?;
    match catchup_result {
        RequestResult::Reply(txns) => {
            debug!("Catchup completed {:?}", meta);
            new_txns.extend(txns);
            let new_root = new_txns.root_hash()?;
            if new_root != target_mt_root {
                return Err(err_msg(
                    VdrErrorKind::Unexpected,
                    "Merkle tree root does not match for new transactions",
                ));
            }
            let json_txns = new_txns.encode_json()?;
            let reload_txns = PoolTransactions::from_json_transactions(json_txns)?;
            if new_txns != reload_txns {
                return Err(err_msg(
                    VdrErrorKind::Unexpected,
                    "Error validating round-trip for pool transactions",
                ));
            }
            Ok((new_txns, meta))
        }
        RequestResult::Failed(err) => {
            trace!("Catchup failed {:?}", meta);
            Err(err)
        }
    }
}

/// Fetch a ledger transaction
pub async fn perform_get_txn<T: Pool>(
    pool: &T,
    ledger_type: i32,
    seq_no: i32,
) -> VdrResult<(RequestResult<String>, RequestResultMeta)> {
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
) -> VdrResult<(RequestResult<NodeReplies<String>>, RequestResultMeta)> {
    let mut request = pool.create_request(req_id, req_json).await?;
    handle_full_request(&mut request, node_aliases, timeout).await
}

/// Dispatch a prepared ledger request to the appropriate handler
pub async fn perform_ledger_request<T: Pool>(
    pool: &T,
    prepared: &PreparedRequest,
) -> VdrResult<(RequestResult<String>, RequestResultMeta)> {
    let mut request = pool
        .create_request(prepared.req_id.clone(), prepared.req_json.to_string())
        .await?;

    let (sp_key, sp_timestamps, is_read_req, sp_parser) = match &prepared.method {
        RequestMethod::Full {
            node_aliases,
            timeout,
        } => {
            let (result, meta) =
                handle_full_request(&mut request, node_aliases.clone(), *timeout).await?;
            return Ok((result.map_result(format_full_reply)?, meta));
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
