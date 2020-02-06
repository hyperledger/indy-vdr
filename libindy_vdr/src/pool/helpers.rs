use std::iter::FromIterator;

use serde_json;

use super::genesis::{parse_transaction_from_json, transactions_to_json};
use super::handlers::{
    build_pool_catchup_request, build_pool_status_request, handle_catchup_request,
    handle_consensus_request, handle_full_request, handle_status_request, CatchupTarget,
    NodeReplies,
};
use super::pool::Pool;
use super::requests::{RequestResult, RequestTarget, TimingResult};

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::ledger::PreparedRequest;
use crate::utils::base58::ToBase58;

pub async fn perform_pool_status_request<T: Pool>(
    pool: &T,
    merkle_tree: MerkleTree,
) -> VdrResult<(RequestResult<Option<CatchupTarget>>, Option<TimingResult>)> {
    let (mt_root, mt_size) = (merkle_tree.root_hash(), merkle_tree.count());
    let message = build_pool_status_request(mt_root, mt_size, pool.get_config().protocol_version)?;
    let req_json = message.serialize()?.to_string();
    let request = pool.create_request("".to_string(), req_json).await?;
    handle_status_request(request, merkle_tree).await
}

pub async fn perform_pool_catchup_request<T: Pool>(
    pool: &T,
    merkle_tree: MerkleTree,
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
) -> VdrResult<(RequestResult<Vec<Vec<u8>>>, Option<TimingResult>)> {
    let message = build_pool_catchup_request(merkle_tree.count(), target_mt_size)?;
    let req_json = message.serialize()?.to_string();
    let request = pool.create_request("".to_string(), req_json).await?;
    handle_catchup_request(request, merkle_tree, target_mt_root, target_mt_size).await
}

pub async fn perform_refresh<T: Pool>(
    pool: &T,
) -> VdrResult<(Option<Vec<String>>, Option<TimingResult>)> {
    let merkle_tree = pool.get_merkle_tree().clone();
    let (result, timing) = perform_pool_status_request(pool, merkle_tree.clone()).await?;
    trace!("Got status result: {:?}", &result);
    match result {
        RequestResult::Reply(target) => match target {
            Some((target_mt_root, target_mt_size)) => {
                info!(
                    "Catchup target found {} {} {:?}",
                    target_mt_root.to_base58(),
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

pub async fn perform_catchup<T: Pool>(
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
            let json_txns = transactions_to_json(txns)?;
            for (idx, txn) in json_txns.iter().enumerate() {
                if parse_transaction_from_json(txn)? != txns[idx] {
                    return Err(err_msg(
                        VdrErrorKind::Unexpected,
                        format!("Error validating rount-trip for pool transaction: {}", txn),
                    ));
                }
            }
            Ok((json_txns, timing))
        }
        RequestResult::Failed(err) => {
            trace!("Catchup failed {:?}", timing);
            Err(err)
        }
    }
}

pub async fn perform_get_txn<T: Pool>(
    pool: &T,
    ledger_type: i32,
    seq_no: i32,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    let builder = pool.get_request_builder();
    let prepared = builder.build_get_txn_request(ledger_type, seq_no, None)?;
    perform_ledger_request(pool, prepared, None).await
}

/* testing only
pub async fn perform_get_validator_info<T: Pool>(
    pool: &T,
) -> LedgerResult<(RequestResult<String>, Option<TimingResult>)> {
    let builder = pool.get_request_builder();
    let did = DidValue::new("V4SGRU86Z58d6TV7PBUe6f", None);
    let mut prepared = builder.build_get_validator_info_request(&did)?;
    prepared.sign(b"000000000000000000000000Trustee1")?;
    trace!("{}", prepared.req_json);
    perform_ledger_request(pool, prepared, Some(RequestTarget::Full(None, None))).await
}
*/

pub async fn perform_ledger_request<T: Pool>(
    pool: &T,
    prepared: PreparedRequest,
    target: Option<RequestTarget>,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    let request = pool
        .create_request(prepared.req_id, prepared.req_json.to_string())
        .await?;
    match target {
        Some(RequestTarget::Full(node_aliases, timeout)) => {
            let (result, timing) = handle_full_request(request, node_aliases, timeout).await?;
            Ok((result.map_result(format_full_reply)?, timing))
        }
        _ => {
            handle_consensus_request(
                request,
                prepared.sp_key,
                prepared.sp_timestamps,
                prepared.is_read_request,
            )
            .await
        }
    }
}

pub fn format_full_reply(replies: NodeReplies<String>) -> VdrResult<String> {
    serde_json::to_string(&serde_json::Map::from_iter(replies.iter().map(
        |(node_alias, reply)| {
            (
                node_alias.clone(),
                serde_json::Value::from(reply.to_string()),
            )
        },
    )))
    .with_input_err("Error serializing response")
}
