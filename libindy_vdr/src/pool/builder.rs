use std::collections::HashMap;

use crate::common::error::prelude::*;
use crate::config::PoolConfig;

use super::genesis::PoolTransactions;
use super::manager::{LocalPool, SharedPool};
use super::networker::{MakeLocal, MakeShared, ZMQNetworkerFactory};
use super::runner::PoolRunner;

/// A utility class for building a new pool instance or runner.
#[derive(Clone)]
pub struct PoolBuilder {
    pub config: PoolConfig,
    transactions: PoolTransactions,
    node_weights: Option<HashMap<String, f32>>,
    refreshed: bool,
}

impl PoolBuilder {
    /// Create a new `PoolBuilder` instance.
    pub fn new(config: PoolConfig, transactions: PoolTransactions) -> Self {
        Self {
            config,
            transactions,
            node_weights: None,
            refreshed: false,
        }
    }

    /// Enable or disable the fast refresh option.
    pub fn refreshed(mut self, flag: bool) -> Self {
        self.refreshed = flag;
        self
    }

    /// Set the node weights associated with the builder.
    pub fn node_weights(mut self, node_weights: Option<HashMap<String, f32>>) -> Self {
        self.node_weights = node_weights;
        self
    }

    /// Create a `LocalPool` instance from the builder, for use in a single thread.
    pub fn into_local(self) -> VdrResult<LocalPool> {
        let merkle_tree = self.transactions.merkle_tree()?;
        LocalPool::build(
            self.config,
            merkle_tree,
            MakeLocal(ZMQNetworkerFactory {}),
            self.node_weights,
            self.refreshed,
        )
    }

    /// Create a `SharedPool` instance from the builder, for use across multiple threads.
    pub fn into_shared(self) -> VdrResult<SharedPool> {
        let merkle_tree = self.transactions.merkle_tree()?;

        SharedPool::build(
            self.config,
            merkle_tree,
            MakeShared(ZMQNetworkerFactory {}),
            self.node_weights,
            self.refreshed,
        )
    }

    /// Create a `PoolRunner` instance from the builder, to handle pool interaction
    /// in a dedicated thread.
    pub fn into_runner(self) -> VdrResult<PoolRunner> {
        let merkle_tree = self.transactions.merkle_tree()?;
        Ok(PoolRunner::new(
            self.config,
            merkle_tree,
            MakeLocal(ZMQNetworkerFactory {}),
            self.node_weights,
            self.refreshed,
        ))
    }
}
