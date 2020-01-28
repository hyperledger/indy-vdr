use std::path::PathBuf;

use super::genesis::{build_merkle_tree, read_transactions};
use super::networker::ZMQNetworker;
use super::pool::{LocalPool, SharedPool};
use super::types::ProtocolVersion;

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::config::PoolConfig;
use crate::utils::validation::Validatable;

#[derive(Debug)]
pub struct PoolFactory {
    pub config: PoolConfig,
    pub merkle_tree: MerkleTree,
    pub transactions: Vec<String>,
}

impl PoolFactory {
    pub fn from_transactions(transactions: &[String]) -> LedgerResult<PoolFactory> {
        let (merkle_tree, transactions) = _load_transactions(transactions)?;
        Ok(PoolFactory {
            config: PoolConfig::default(),
            merkle_tree,
            transactions,
        })
    }

    pub fn from_genesis_file(genesis_file: &str) -> LedgerResult<PoolFactory> {
        Self::from_genesis_path(&PathBuf::from(genesis_file))
    }

    pub fn from_genesis_path(genesis_path: &PathBuf) -> LedgerResult<PoolFactory> {
        // FIXME convert into config error
        let txns = read_transactions(genesis_path)?;
        trace!("Loaded transactions from {:?}", genesis_path);
        if txns.len() == 0 {
            return Err((LedgerErrorKind::Config, "Empty genesis transaction file").into());
        }
        Self::from_transactions(&txns)
    }

    pub fn get_config(&self) -> PoolConfig {
        return self.config;
    }

    pub fn set_config(&mut self, config: PoolConfig) -> LedgerResult<()> {
        // FIXME convert into config error
        config.validate()?;
        self.config = config;
        Ok(())
    }

    pub fn get_protocol_version(&self) -> ProtocolVersion {
        self.config.protocol_version
    }

    pub fn set_protocol_version(&mut self, version: ProtocolVersion) {
        self.config.protocol_version = version
    }

    pub fn get_transactions(self) -> Vec<String> {
        self.transactions
    }

    pub fn add_transactions(&mut self, new_txns: &[String]) -> LedgerResult<()> {
        let mut txns = self.transactions.clone();
        txns.extend_from_slice(new_txns);
        self.set_transactions(&txns)
    }

    pub fn set_transactions(&mut self, transactions: &[String]) -> LedgerResult<()> {
        let (merkle_tree, transactions) = _load_transactions(transactions)?;
        self.merkle_tree = merkle_tree;
        self.transactions = transactions;
        Ok(())
    }

    pub fn create_local(&self) -> LedgerResult<LocalPool> {
        LocalPool::build::<ZMQNetworker>(self.config, self.merkle_tree.clone(), None)
    }

    pub fn create_shared(&self) -> LedgerResult<SharedPool> {
        SharedPool::build::<ZMQNetworker>(self.config, self.merkle_tree.clone(), None)
    }
}

fn _load_transactions(transactions: &[String]) -> LedgerResult<(MerkleTree, Vec<String>)> {
    if transactions.len() == 0 {
        return Err((LedgerErrorKind::Config, "No genesis transactions").into());
    }
    // FIXME convert into config error
    let merkle_tree = build_merkle_tree(transactions)?;
    Ok((merkle_tree, transactions.to_vec()))
}
