use std::io::BufRead;
use std::path::PathBuf;
use std::{fs, io};

use serde_json;
use serde_json::Value as SJsonValue;

use crate::domain::ledger::request::{ProtocolVersion, DEFAULT_PROTOCOL_VERSION};
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

#[derive(Debug)]
pub struct PoolFactory {
    pub merkle_tree: MerkleTree,
    pub protocol_version: ProtocolVersion,
}

impl PoolFactory {
    pub fn new() -> LedgerResult<PoolFactory> {
        let tree = MerkleTree::from_vec(Vec::new())?;
        Ok(PoolFactory {
            merkle_tree: tree,
            protocol_version: DEFAULT_PROTOCOL_VERSION,
        })
    }

    pub fn from_genesis_file(genesis_file: &str) -> LedgerResult<PoolFactory> {
        Self::from_genesis_path(&PathBuf::from(genesis_file))
    }

    pub fn from_genesis_path(genesis_path: &PathBuf) -> LedgerResult<PoolFactory> {
        let tree = _merkle_tree_from_genesis(genesis_path)?;
        Ok(PoolFactory {
            merkle_tree: tree,
            protocol_version: DEFAULT_PROTOCOL_VERSION,
        })
    }

    pub fn set_protocol_version(&mut self, version: ProtocolVersion) {
        self.protocol_version = version
    }
}

fn _merkle_tree_from_genesis(file_name: &PathBuf) -> LedgerResult<MerkleTree> {
    let mut mt = MerkleTree::from_vec(Vec::new())?;

    let f = fs::File::open(file_name)
        .to_result(LedgerErrorKind::IOError, "Can't open genesis txn file")?;

    let reader = io::BufReader::new(&f);

    for line in reader.lines() {
        let line: String =
            line.to_result(LedgerErrorKind::IOError, "Can't read from genesis txn file")?;

        if line.trim().is_empty() {
            continue;
        };
        mt.append(_parse_txn_from_json(&line)?)?;
    }

    Ok(mt)
}

fn _parse_txn_from_json(txn: &str) -> LedgerResult<Vec<u8>> {
    let txn = txn.trim();

    if txn.is_empty() {
        return Ok(vec![]);
    }

    let txn: SJsonValue = serde_json::from_str(txn).to_result(
        LedgerErrorKind::InvalidStructure,
        "Genesis txn is mailformed json",
    )?;

    rmp_serde::encode::to_vec_named(&txn).to_result(
        LedgerErrorKind::InvalidState,
        "Can't encode genesis txn as message pack",
    )
}
