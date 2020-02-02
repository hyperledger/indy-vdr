use std::rc::Rc;
use std::thread;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::executor::block_on;
use futures::select;
use futures::stream::{FuturesUnordered, StreamExt};

use super::helpers::perform_ledger_request;
use super::networker::{Networker, NetworkerFactory};
use super::requests::{RequestResult, TimingResult};
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

    pub fn get_transactions(&self, callback: TxnCallback) -> VdrResult<()> {
        self.send_event(PoolEvent::GetTransactions(callback))
    }

    pub fn send_request(&self, request: PreparedRequest, callback: ReqCallback) -> VdrResult<()> {
        self.send_event(PoolEvent::SendRequest(request, callback))
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

type ReqCallback = Box<dyn (FnOnce(ReqResponse) -> ()) + Send>;

type ReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

type TxnCallback = Box<dyn (FnOnce(TxnResponse) -> ()) + Send>;

type TxnResponse = VdrResult<Vec<String>>;

enum PoolEvent {
    GetTransactions(TxnCallback),
    SendRequest(PreparedRequest, ReqCallback),
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
        let mut requests = FuturesUnordered::new();
        let receiver = &mut self.receiver;
        loop {
            select! {
                recv_evt = receiver.next() => {
                    match recv_evt {
                        Some(PoolEvent::GetTransactions(callback)) => {
                            let txns = self.pool.get_transactions();
                            callback(txns);
                        }
                        Some(PoolEvent::SendRequest(request, callback)) => {
                            let fut = _perform_ledger_request(&self.pool, request, callback);
                            requests.push(fut);
                        }
                        None => { trace!("Pool runner sender dropped") }
                    }
                }
                req_evt = requests.next() => {
                    match req_evt {
                        Some(()) => trace!("Request response dispatched"),
                        None => trace!("No pending requests")
                    }
                }
                complete => break
            }
        }
    }
}

async fn _perform_ledger_request(
    pool: &LocalPool,
    request: PreparedRequest,
    callback: ReqCallback,
) {
    let result = perform_ledger_request(pool, request, None).await;
    callback(result);
}
