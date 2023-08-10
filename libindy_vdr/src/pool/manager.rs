extern crate rand;
extern crate rmp_serde;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use futures_channel::mpsc::unbounded;
use futures_util::future::{lazy, FutureExt, LocalBoxFuture};
use rand::seq::SliceRandom;

use super::genesis::{build_node_transaction_map, build_verifiers, PoolTransactions};
use super::networker::{
    LocalNetworker, Networker, NetworkerEvent, NetworkerFactory, SharedNetworker,
};
use super::requests::{PoolRequest, PoolRequestImpl};
use super::types::{PoolSetup, RequestHandle, Verifiers};

use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::config::PoolConfig;
use crate::ledger::RequestBuilder;
use crate::utils::base58;

/// A generic verifier pool with support for creating pool transaction requests
pub trait Pool: Clone {
    type Request: PoolRequest;

    /// Get the pool configuration for this instance
    fn get_config(&self) -> &PoolConfig;

    /// Create a new pool request instance
    fn create_request(
        &self,
        req_id: String,
        req_json: String,
    ) -> LocalBoxFuture<'_, VdrResult<Self::Request>>;

    /// Get the merkle tree representing the verifier pool transactions
    fn get_merkle_tree(&self) -> &MerkleTree;

    /// A sequence of verifier node aliases
    fn get_node_aliases(&self) -> Vec<String>;

    /// Get the size and root of the pool transactions merkle tree
    fn get_merkle_tree_info(&self) -> (String, usize) {
        let tree = self.get_merkle_tree();
        (base58::encode(tree.root_hash()), tree.count())
    }

    /// Get a request builder corresponding to this verifier pool
    fn get_request_builder(&self) -> RequestBuilder {
        RequestBuilder::new(self.get_config().protocol_version)
    }

    /// Get the set of verifier pool transactions in JSON format
    fn get_json_transactions(&self) -> VdrResult<Vec<String>> {
        PoolTransactions::from(self.get_merkle_tree()).encode_json()
    }

    /// Get the summarized verifier details.
    fn get_verifier_info(&self) -> VdrResult<Verifiers>;
}

/// The default `Pool` implementation
#[derive(Clone)]
pub struct PoolImpl<S: AsRef<PoolSetup> + Clone, T: Networker + Clone> {
    setup: S,
    networker: T,
}

/// A verifier pool instance restricted to a single thread
pub type LocalPool = PoolImpl<Rc<PoolSetup>, LocalNetworker>;

/// A verifier pool instance which can be shared between threads
pub type SharedPool = PoolImpl<Arc<PoolSetup>, SharedNetworker>;

impl<S, T> PoolImpl<S, T>
where
    S: AsRef<PoolSetup> + Clone + From<Box<PoolSetup>>,
    T: Networker + Clone,
{
    pub(crate) fn new(setup: S, networker: T) -> Self {
        Self { setup, networker }
    }

    /// Build a new verifier pool instance
    pub fn build<F>(
        config: PoolConfig,
        merkle_tree: MerkleTree,
        networker_factory: F,
        node_weights: Option<HashMap<String, f32>>,
    ) -> VdrResult<Self>
    where
        F: NetworkerFactory<Output = T>,
    {
        let txn_map = build_node_transaction_map(&merkle_tree, config.protocol_version)?;
        let verifiers = build_verifiers(txn_map)?;
        let networker = networker_factory.make_networker(config.clone(), &verifiers)?;
        let setup = PoolSetup::new(config, merkle_tree, node_weights, verifiers);
        Ok(Self::new(S::from(Box::new(setup)), networker))
    }
}

impl<S, T> Pool for PoolImpl<S, T>
where
    S: AsRef<PoolSetup> + Clone,
    T: Networker + Clone,
{
    type Request = PoolRequestImpl<S, T>;

    fn create_request(
        &self,
        req_id: String,
        req_json: String,
    ) -> LocalBoxFuture<'_, VdrResult<Self::Request>> {
        let setup = self.setup.clone();
        let networker = self.networker.clone();
        lazy(move |_| {
            let (tx, rx) = unbounded();
            let handle = RequestHandle::next();
            let setup_ref = setup.as_ref();
            let node_order = choose_nodes(&setup_ref.verifiers, setup_ref.node_weights.as_ref());
            debug!(
                "New {}: reqId({}), node order: {:?}",
                handle, req_id, node_order
            );
            networker.send(NetworkerEvent::NewRequest(handle, req_id, req_json, tx))?;
            Ok(PoolRequestImpl::new(
                handle, rx, setup, networker, node_order,
            ))
        })
        .boxed_local()
    }

    fn get_config(&self) -> &PoolConfig {
        &self.setup.as_ref().config
    }

    fn get_merkle_tree(&self) -> &MerkleTree {
        &self.setup.as_ref().merkle_tree
    }

    fn get_node_aliases(&self) -> Vec<String> {
        self.setup.as_ref().verifiers.keys().cloned().collect()
    }

    fn get_verifier_info(&self) -> VdrResult<Verifiers> {
        Ok(self.setup.as_ref().verifiers.clone())
    }
}

pub(crate) fn choose_nodes(
    verifiers: &Verifiers,
    weights: Option<&HashMap<String, f32>>,
) -> Vec<String> {
    let mut weighted = verifiers
        .keys()
        .filter_map(|name| {
            let weight = weights
                .as_ref()
                .and_then(|w| w.get(name))
                .copied()
                .unwrap_or(1.0);
            if weight <= 0.0 {
                None
            } else {
                Some((weight, name.as_str()))
            }
        })
        .collect::<Vec<(f32, &str)>>();
    let mut rng = rand::thread_rng();
    let mut result = vec![];
    for _ in 0..weighted.len() {
        let found = weighted
            .choose_weighted_mut(&mut rng, |item| item.0)
            .unwrap();
        found.0 = 0.0;
        result.push(found.1.to_string());
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::pool::{VerifierInfo, Verifiers};

    use super::*;

    #[test]
    fn test_choose_nodes() {
        let test_verif = VerifierInfo {
            client_addr: "127.0.0.1".into(),
            node_addr: "127.0.0.1".into(),
            public_key: "pk".into(),
            enc_key: "ek".into(),
            bls_key: None,
        };
        let mut verifiers = Verifiers::new();
        verifiers.insert("a".into(), test_verif.clone());
        verifiers.insert("b".into(), test_verif.clone());
        verifiers.insert("c".into(), test_verif);

        let mut weights = HashMap::new();
        weights.insert("a".into(), 0.0);
        weights.insert("b".into(), 0.000001);
        let found = choose_nodes(&verifiers, Some(&weights));
        assert_eq!(found, ["c", "b"]);
    }
}
