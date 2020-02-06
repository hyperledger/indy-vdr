use std::rc::Rc;
use std::thread;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::executor::block_on;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{select, FutureExt};

use super::helpers::{perform_ledger_request, perform_refresh};
use super::networker::{Networker, NetworkerFactory};
use super::requests::{RequestResult, RequestTarget, TimingResult};
use super::{LocalPool, Pool};
use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::config::PoolConfig;
use crate::ledger::PreparedRequest;

pub struct PoolRunner {
    sender: Option<UnboundedSender<PoolEvent>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl PoolRunner {
    pub fn new<F>(config: PoolConfig, merkle_tree: MerkleTree, networker_factory: F) -> Self
    where
        F: NetworkerFactory<Output = Rc<dyn Networker>> + Send + 'static,
    {
        let (sender, receiver) = unbounded();
        let worker = thread::spawn(move || {
            // FIXME handle error on build
            let pool = LocalPool::build(config, merkle_tree, networker_factory, None).unwrap();
            let mut thread = PoolThread::new(pool, receiver);
            thread.run();
            debug!("Pool thread ended")
        });
        Self {
            sender: Some(sender),
            worker: Some(worker),
        }
    }

    pub fn get_transactions(&self, callback: GetTxnsCallback) -> VdrResult<()> {
        self.send_event(PoolEvent::GetTransactions(callback))
    }

    pub fn refresh(&self, callback: RefreshCallback) -> VdrResult<()> {
        self.send_event(PoolEvent::Refresh(callback))
    }

    pub fn send_request(
        &self,
        request: PreparedRequest,
        target: Option<RequestTarget>,
        callback: SendReqCallback,
    ) -> VdrResult<()> {
        self.send_event(PoolEvent::SendRequest(request, target, callback))
    }

    fn send_event(&self, event: PoolEvent) -> VdrResult<()> {
        // FIXME error should indicate that the thread exited, so indicate such in result
        if let Some(sender) = &self.sender {
            sender
                .unbounded_send(event)
                .map_err(|_| err_msg(VdrErrorKind::Unexpected, "Error sending to pool thread"))
        } else {
            // FIXME error kind
            Err(err_msg(VdrErrorKind::Unexpected, "Pool is closed"))
        }
    }

    pub fn close(&mut self) -> bool {
        if self.sender.is_none() {
            return false;
        } else {
            drop(self.sender.take());
            return true;
        }
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

type GetTxnsCallback = Box<dyn (FnOnce(GetTxnsResponse) -> ()) + Send>;

type GetTxnsResponse = VdrResult<Vec<String>>;

type RefreshCallback = Box<dyn (FnOnce(RefreshResponse) -> ()) + Send>;

type RefreshResponse = VdrResult<(Vec<String>, Option<Vec<String>>, Option<TimingResult>)>;

type SendReqCallback = Box<dyn (FnOnce(SendReqResponse) -> ()) + Send>;

type SendReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

enum PoolEvent {
    GetTransactions(GetTxnsCallback),
    Refresh(RefreshCallback),
    SendRequest(PreparedRequest, Option<RequestTarget>, SendReqCallback),
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
                        Some(PoolEvent::GetTransactions(callback)) => {
                            let txns = self.pool.get_transactions();
                            callback(txns);
                        }
                        Some(PoolEvent::Refresh(callback)) => {
                            let fut = _perform_refresh(&self.pool, callback);
                            futures.push(fut.boxed_local());
                        }
                        Some(PoolEvent::SendRequest(request, target, callback)) => {
                            let fut = _perform_ledger_request(&self.pool, request, target, callback);
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

async fn _perform_refresh(pool: &LocalPool, callback: RefreshCallback) {
    let result = {
        match perform_refresh(pool).await {
            Ok((new_txns, timing)) => match pool.get_transactions() {
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
    target: Option<RequestTarget>,
    callback: SendReqCallback,
) {
    let result = perform_ledger_request(pool, request, target).await;
    callback(result);
}
