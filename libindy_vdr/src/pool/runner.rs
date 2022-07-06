use std::collections::HashMap;
use std::rc::Rc;
use std::thread;

use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_executor::block_on;
use futures_util::stream::{FuturesUnordered, StreamExt};
use futures_util::{select, FutureExt};

use super::helpers::{perform_ledger_request, perform_refresh};
use super::networker::{Networker, NetworkerFactory};
use super::requests::PreparedRequest;
use super::types::{RequestResult, TimingResult, Verifiers};
use super::{LocalPool, Pool};
use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::config::PoolConfig;
use crate::utils::base58;

/// The `PoolRunner` instance creates a separate thread for handling pool events,
/// allowing the use of callbacks instead of async functions for interacting
/// with the pool as well as simplifying validator pool refreshes.
pub struct PoolRunner {
    sender: Option<UnboundedSender<PoolEvent>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl PoolRunner {
    /// Create a new `PoolRunner` instance and run the associated worker thread.
    pub fn new<F>(
        config: PoolConfig,
        merkle_tree: MerkleTree,
        networker_factory: F,
        node_weights: Option<HashMap<String, f32>>,
    ) -> Self
    where
        F: NetworkerFactory<Output = Rc<dyn Networker>> + Send + 'static,
    {
        let (sender, receiver) = unbounded();
        let worker = thread::spawn(move || {
            // FIXME handle error on build
            let pool =
                LocalPool::build(config.clone(), merkle_tree, networker_factory, node_weights)
                    .unwrap();
            let mut thread = PoolThread::new(pool, receiver);
            thread.run();
            debug!("Pool thread ended")
        });
        Self {
            sender: Some(sender),
            worker: Some(worker),
        }
    }

    /// Fetch the status of the pool instance.
    pub fn get_status(&self, callback: Callback<GetStatusResponse>) -> VdrResult<()> {
        self.send_event(PoolEvent::GetStatus(callback))
    }

    /// Fetch the current set of pool transactions.
    pub fn get_transactions(&self, callback: Callback<GetTxnsResponse>) -> VdrResult<()> {
        self.send_event(PoolEvent::GetTransactions(callback))
    }

    /// Fetch the current set of pool transactions.
    pub fn get_verifiers(&self, callback: Callback<GetVerifiersResponse>) -> VdrResult<()> {
        self.send_event(PoolEvent::GetVerifiers(callback))
    }

    /// Fetch the latest pool transactions and switch to the new validator
    /// pool if necessary.
    pub fn refresh(&self, callback: Callback<RefreshResponse>) -> VdrResult<()> {
        self.send_event(PoolEvent::Refresh(callback))
    }

    /// Submit a request to the validator pool.
    pub fn send_request(
        &self,
        request: PreparedRequest,
        callback: Callback<SendReqResponse>,
    ) -> VdrResult<()> {
        self.send_event(PoolEvent::SendRequest(request, callback))
    }

    /// Send an event to the worker thread.
    fn send_event(&self, event: PoolEvent) -> VdrResult<()> {
        // FIXME error should indicate that the thread exited, so indicate such in result
        if let Some(sender) = &self.sender {
            sender
                .unbounded_send(event)
                .map_err(|_| err_msg(VdrErrorKind::Unexpected, "Error sending to pool thread"))
        } else {
            Err(err_msg(VdrErrorKind::Unexpected, "Pool is closed"))
        }
    }

    /// Shut down the associated worker thread and release any pool resources.
    pub fn close(&mut self) -> bool {
        self.sender.take().is_some()
    }
}

impl Drop for PoolRunner {
    fn drop(&mut self) {
        self.close();
        if let Some(worker) = self.worker.take() {
            debug!("Drop pool runner thread");
            worker.join().unwrap()
        }
    }
}

type Callback<R> = Box<dyn FnOnce(R) + Send>;

type GetStatusResponse = VdrResult<PoolRunnerStatus>;

type GetTxnsResponse = VdrResult<Vec<String>>;

type GetVerifiersResponse = VdrResult<Verifiers>;

type RefreshResponse = VdrResult<(Vec<String>, Option<Vec<String>>, Option<TimingResult>)>;

type SendReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

enum PoolEvent {
    GetStatus(Callback<GetStatusResponse>),
    GetTransactions(Callback<GetTxnsResponse>),
    GetVerifiers(Callback<GetVerifiersResponse>),
    Refresh(Callback<RefreshResponse>),
    SendRequest(PreparedRequest, Callback<SendReqResponse>),
}

/// The current status of a validator pool.
#[derive(Serialize)]
pub struct PoolRunnerStatus {
    /// The root hash of the merkle tree
    pub mt_root: String,
    /// The number of transactions
    pub mt_size: usize,
    /// The aliases of the validator nodes
    pub nodes: Vec<String>,
}

impl PoolRunnerStatus {
    pub fn serialize(&self) -> VdrResult<String> {
        Ok(serde_json::to_value(self)
            .with_err_msg(VdrErrorKind::Unexpected, "Error serializing pool status")?
            .to_string())
    }
}

struct PoolThread {
    pool: LocalPool,
    receiver: UnboundedReceiver<PoolEvent>,
}

impl PoolThread {
    fn new(pool: LocalPool, receiver: UnboundedReceiver<PoolEvent>) -> Self {
        Self { pool, receiver }
    }

    fn run(&mut self) {
        block_on(self.run_loop())
    }

    async fn run_loop(&mut self) {
        let mut futures = FuturesUnordered::new();
        let receiver = &mut self.receiver;
        loop {
            select! {
                recv_evt = receiver.next() => {
                    match recv_evt {
                        Some(PoolEvent::GetStatus(callback)) => {
                            let tree = self.pool.get_merkle_tree();
                            let status = PoolRunnerStatus {
                                mt_root: base58::encode(tree.root_hash()),
                                mt_size: tree.count(),
                                nodes: self.pool.get_node_aliases(),
                            };
                            callback(Ok(status));
                        }
                        Some(PoolEvent::GetTransactions(callback)) => {
                            let txns = self.pool.get_json_transactions();
                            callback(txns);
                        }
                        Some(PoolEvent::GetVerifiers(callback)) => {
                            let vers = self.pool.get_verifier_info();
                            callback(vers);
                        }
                        Some(PoolEvent::Refresh(callback)) => {
                            let fut = _perform_refresh(&self.pool, callback);
                            futures.push(fut.boxed_local());
                        }
                        Some(PoolEvent::SendRequest(request, callback)) => {
                            let fut = _perform_ledger_request(&self.pool, request, callback);
                            futures.push(fut.boxed_local());
                        }
                        None => { trace!("Pool runner sender dropped") }
                    }
                }
                req_evt = futures.next() => {
                    match req_evt {
                        Some(()) => trace!("Callback response dispatched"),
                        None => trace!("No pending callbacks")
                    }
                }
                complete => break
            }
        }
    }
}

async fn _perform_refresh(pool: &LocalPool, callback: Callback<RefreshResponse>) {
    let result = {
        match perform_refresh(pool).await {
            Ok((new_txns, timing)) => match pool.get_json_transactions() {
                Ok(old_txns) => Ok((old_txns, new_txns, timing)),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    };
    callback(result);
}

async fn _perform_ledger_request(
    pool: &LocalPool,
    request: PreparedRequest,
    callback: Callback<SendReqResponse>,
) {
    let result = perform_ledger_request(pool, &request).await;
    callback(result);
}
