use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use serde_json::{self, Value as SJsonValue};

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::utils::{base58, ValidationError};

use super::requests::{PoolRequest, RequestEvent};
use super::types::{
    self, CatchupReq, LedgerStatus, LedgerType, Message, NodeReplies, ProtocolVersion,
    RequestResult, SingleReply, TimingResult,
};

mod catchup;
mod consensus;
mod full;
mod status;

pub use catchup::handle_catchup_request;
pub use consensus::handle_consensus_request;
pub use full::handle_full_request;
pub use status::{handle_status_request, CatchupTarget};

#[derive(Debug)]
struct ReplyState<T> {
    pub inner: HashMap<String, SingleReply<T>>,
}

#[derive(Default)]
struct ReplyCounts {
    pub replies: usize,
    pub failed: usize,
    pub timeout: usize,
}

impl<T> ReplyState<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn add_reply(&mut self, node_alias: String, reply: T) {
        self.inner.insert(node_alias, SingleReply::Reply(reply));
    }

    pub fn add_failed(&mut self, node_alias: String, raw_msg: String) {
        self.inner.insert(node_alias, SingleReply::Failed(raw_msg));
    }

    pub fn add_timeout(&mut self, node_alias: String) {
        self.inner
            .entry(node_alias)
            .or_insert(SingleReply::Timeout());
    }

    pub fn result(self) -> NodeReplies<T> {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn counts(&self) -> ReplyCounts {
        let mut counts = ReplyCounts::default();
        self.inner.values().for_each(|r| {
            match r {
                SingleReply::Reply(_) => counts.replies += 1,
                SingleReply::Failed(_) => counts.failed += 1,
                SingleReply::Timeout() => counts.timeout += 1,
            };
        });
        counts
    }

    #[allow(unused)]
    pub fn failed_len(&self) -> usize {
        self.inner
            .values()
            .filter(|r| matches!(r, SingleReply::Failed(_)))
            .count()
    }

    pub fn sample_failed(&self) -> Option<String> {
        self.inner.values().find_map(|r| {
            if let SingleReply::Failed(msg) = r {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    pub fn get_error(&self) -> VdrError {
        let counts = self.counts();
        if counts.replies > 0 {
            VdrErrorKind::PoolNoConsensus.into()
        } else if counts.failed > 0 {
            let failed = self.sample_failed().unwrap();
            VdrErrorKind::PoolRequestFailed(failed).into()
        } else {
            VdrErrorKind::PoolTimeout.into()
        }
    }
}

#[derive(Debug)]
struct ConsensusState<K: Eq + Hash, T: Eq + Hash> {
    inner: HashMap<K, HashSet<T>>,
}

impl<K: Eq + Hash, T: Eq + Hash> ConsensusState<K, T> {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn max_entry(&self) -> Option<(&K, usize)> {
        self.inner
            .iter()
            .map(|(key, set)| (key, set.len()))
            .max_by_key(|entry| entry.1)
    }

    #[allow(dead_code)]
    fn max_len(&self) -> usize {
        self.max_entry().map(|entry| entry.1).unwrap_or(0)
    }

    pub fn insert(&mut self, key: K, reply: T) -> &mut HashSet<T> {
        let set = self.inner.entry(key).or_insert_with(HashSet::new);
        set.insert(reply);
        set
    }
}

#[derive(Debug)]
pub(crate) struct HashableValue {
    pub inner: SJsonValue,
}

impl Hash for HashableValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // FIXME does to_string produce canonical results??
        serde_json::to_string(&self.inner).unwrap().hash(state); //TODO
    }
}

impl PartialEq for HashableValue {
    fn eq(&self, other: &HashableValue) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for HashableValue {}

fn min_consensus(cnt: usize) -> usize {
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
) -> Result<(), ValidationError> {
    let mut bytes_proofs: Vec<Vec<u8>> = Vec::new();

    for cons_proof in cons_proofs {
        let cons_proof: &String = cons_proof;

        bytes_proofs.push(
            base58::decode(cons_proof)
                .map_err(|_| invalid!("Can't decode node consistency proof"))?,
        );
    }

    if !mt.consistency_proof(target_mt_root, target_mt_size, &bytes_proofs)? {
        return Err(invalid!("Consistency proof verification failed"));
    }

    Ok(())
}

pub(crate) fn build_pool_status_request(
    merkle_root: &[u8],
    merkle_tree_size: usize,
    protocol_version: ProtocolVersion,
) -> VdrResult<Message> {
    let lr = LedgerStatus {
        txnSeqNo: merkle_tree_size,
        merkleRoot: base58::encode(merkle_root),
        ledgerId: LedgerType::POOL as u8,
        ppSeqNo: None,
        viewNo: None,
        protocolVersion: Some(protocol_version as usize),
    };
    Ok(Message::LedgerStatus(lr))
}

pub(crate) fn build_pool_catchup_request(
    from_mt_size: usize,
    target_mt_size: usize,
) -> VdrResult<Message> {
    if from_mt_size >= target_mt_size {
        return Err(input_err("No transactions to catch up"));
    }
    let seq_no_start = from_mt_size + 1;
    let seq_no_end = target_mt_size;

    let cr = CatchupReq {
        ledgerId: LedgerType::POOL as usize,
        seqNoStart: seq_no_start,
        seqNoEnd: seq_no_end,
        catchupTill: target_mt_size,
    };
    Ok(Message::CatchupReq(cr))
}
