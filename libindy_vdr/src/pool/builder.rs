use super::genesis::PoolTransactions;
use super::networker::{MakeLocal, MakeShared, ZMQNetworkerFactory};
use super::pool::{LocalPool, SharedPool};
use super::runner::PoolRunner;

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::config::PoolConfig;

use std::collections::HashMap;

#[derive(Clone)]
pub struct PoolBuilder {
    pub config: PoolConfig,
    merkle_tree: Option<MerkleTree>,
    node_weights: Option<HashMap<String, f32>>,
}

impl PoolBuilder {
    pub fn new(
        config: PoolConfig,
        merkle_tree: Option<MerkleTree>,
        node_weights: Option<HashMap<String, f32>>,
    ) -> Self {
        Self {
            config,
            merkle_tree,
            node_weights,
        }
    }

    pub fn from_config(config: PoolConfig) -> Self {
        Self::new(config, None, None)
    }

    pub fn merkle_tree(mut self, merkle_tree: MerkleTree) -> Self {
        self.merkle_tree.replace(merkle_tree);
        self
    }

    pub fn node_weights(mut self, node_weights: Option<HashMap<String, f32>>) -> Self {
        self.node_weights = node_weights;
        self
    }

    pub fn transactions(mut self, transactions: PoolTransactions) -> VdrResult<Self> {
        let merkle_tree = transactions.into_merkle_tree()?;
        self.merkle_tree.replace(merkle_tree);
        Ok(self)
    }

    pub fn into_local(self) -> VdrResult<LocalPool> {
        if self.merkle_tree.is_none() {
            return Err(err_msg(
                VdrErrorKind::Config,
                "No pool transactions provided",
            ));
        }
        LocalPool::build(
            self.config,
            self.merkle_tree.unwrap(),
            MakeLocal(ZMQNetworkerFactory {}),
            self.node_weights,
        )
    }

    pub fn into_shared(self) -> VdrResult<SharedPool> {
        if self.merkle_tree.is_none() {
            return Err(err_msg(
                VdrErrorKind::Config,
                "No pool transactions provided",
            ));
        }
        SharedPool::build(
            self.config,
            self.merkle_tree.unwrap(),
            MakeShared(ZMQNetworkerFactory {}),
            self.node_weights,
        )
    }

    pub fn into_runner(self) -> VdrResult<PoolRunner> {
        if self.merkle_tree.is_none() {
            return Err(err_msg(
                VdrErrorKind::Config,
                "No pool transactions provided",
            ));
        }
        Ok(PoolRunner::new(
            self.config,
            self.merkle_tree.unwrap(),
            MakeLocal(ZMQNetworkerFactory {}),
            self.node_weights,
        ))
    }
}

impl Default for PoolBuilder {
    fn default() -> Self {
        PoolBuilder::new(PoolConfig::default(), None, None)
    }
}
