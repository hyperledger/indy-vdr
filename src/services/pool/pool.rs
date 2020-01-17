use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures::channel::mpsc::{channel, Receiver};
use futures::future::{lazy, FutureExt, LocalBoxFuture};
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};

use pin_utils::unsafe_pinned;

use crate::domain::did::{DidValue, DEFAULT_LIBINDY_DID};
use crate::domain::ledger::request::{get_request_id, Request};
use crate::domain::ledger::txn::{GetTxnOperation, LedgerType};

use super::genesis::{build_tree, show_transactions};
use super::networker::{Networker, NetworkerEvent, ZMQNetworker};
use super::requests::{
    perform_catchup_request, perform_consensus_request, perform_full_request,
    perform_single_request, perform_status_request, CatchupRequestResult, ConsensusResult,
    RequestEvent, RequestExtEvent, RequestHandle, RequestState, RequestTimeout, RequestTiming,
    SingleReply, StatusRequestResult, TimingResult,
};
use super::types::{Nodes, PoolConfig};

use crate::utils::base58::ToBase58;
use crate::utils::error::prelude::*;

pub async fn connect(config: PoolConfig, txns: Vec<String>) -> LedgerResult<Pool> {
    let pool = ZMQNetworker::create_pool(config, txns.clone(), None)?;
    let merkle_tree = build_tree(&txns)?;
    let result = perform_status_request(&pool, merkle_tree).await?;
    trace!("Got status result: {:?}", &result);
    match result {
        StatusRequestResult::CatchupTargetFound(mt_root, mt_size, timing) => {
            trace!(
                "Catchup target found {} {} {:?}",
                mt_root.to_base58(),
                mt_size,
                timing
            );
            perform_catchup(&pool, txns, mt_root, mt_size).await?;
            Ok(pool)
        }
        StatusRequestResult::CatchupTargetNotFound(err, timing) => {
            trace!("Catchup target not found {:?}", timing);
            Err(err)
        }
        StatusRequestResult::Synced(timing) => {
            trace!("Synced! {:?}", timing);
            Ok(pool)
        }
    }
}

pub async fn perform_catchup(
    pool: &Pool,
    txns: Vec<String>,
    mt_root: Vec<u8>,
    mt_size: usize,
) -> LedgerResult<()> {
    let merkle_tree = build_tree(&txns)?;
    let catchup_result = perform_catchup_request(pool, merkle_tree, mt_root, mt_size).await?;
    trace!("Got catchup result: {:?}", &catchup_result);
    match catchup_result {
        CatchupRequestResult::Synced(txns, timing) => {
            trace!("Catchup synced! {:?}", timing);
            let txns = show_transactions(&txns, pool.config().protocol_version)?;
            for txn in txns {
                print!("{}\n", txn);
            }
            Ok(())
        }
        CatchupRequestResult::Timeout() => {
            trace!("Catchup timeout");
            Err(err_msg(
                LedgerErrorKind::PoolTimeout,
                "Timeout on catchup request",
            ))
        }
    }
}

fn _build_get_txn_request(
    ledger_type: i32,
    seq_no: i32,
    identifier: Option<&DidValue>,
    protocol_version: Option<usize>,
) -> LedgerResult<(String, String)> {
    if seq_no <= 0 {
        return Err(err_msg(
            LedgerErrorKind::InvalidStructure,
            "Transaction number must be > 0",
        ));
    }
    let operation = GetTxnOperation::new(seq_no, ledger_type);
    let req_id = get_request_id();
    let identifier = identifier.or(Some(&DEFAULT_LIBINDY_DID));
    let body = Request::build_request(req_id, operation, identifier, protocol_version)
        .map_err(|err| err_msg(LedgerErrorKind::InvalidStructure, err))?;
    Ok((format!("{}", req_id), body))
}

pub async fn perform_get_txn(
    pool: &Pool,
    ledger_type: LedgerType,
    seq_no: i32,
) -> LedgerResult<(String, TimingResult)> {
    let (req_id, message) = _build_get_txn_request(
        ledger_type.to_id(),
        seq_no,
        None,
        Some(pool.config().protocol_version.to_id()),
    )?;
    trace!("{} {}", req_id, message);
    let result = perform_single_request(pool, &req_id, &message, None, (None, None)).await?;
    match result {
        ConsensusResult::Reply(message, timing) => {
            trace!("Got request response {} {:?}", &message, timing);
            Ok((message, timing.unwrap()))
        }
        ConsensusResult::NoConsensus(timing) => {
            trace!("No consensus {:?}", timing);
            Err(err_msg(LedgerErrorKind::PoolTimeout, "No consensus"))
        }
    }
}

// FIXME testing only
pub async fn perform_get_txn_consensus(
    pool: &Pool,
    ledger_type: LedgerType,
    seq_no: i32,
) -> LedgerResult<(String, TimingResult)> {
    let (req_id, message) = _build_get_txn_request(
        ledger_type.to_id(),
        seq_no,
        None,
        Some(pool.config().protocol_version.to_id()),
    )?;
    trace!("{} {}", req_id, message);
    let result = perform_consensus_request(pool, &req_id, &message).await?;
    match result {
        ConsensusResult::Reply(message, timing) => {
            trace!("Got request response {} {:?}", &message, timing);
            Ok((message, timing.unwrap()))
        }
        ConsensusResult::NoConsensus(timing) => {
            trace!("No consensus {:?}", timing);
            Err(err_msg(LedgerErrorKind::PoolTimeout, "No consensus"))
        }
    }
}

// FIXME testing only
pub async fn perform_get_txn_full(
    pool: &Pool,
    ledger_type: LedgerType,
    seq_no: i32,
) -> LedgerResult<(String, TimingResult)> {
    let (req_id, message) = _build_get_txn_request(
        ledger_type.to_id(),
        seq_no,
        None,
        Some(pool.config().protocol_version.to_id()),
    )?;
    trace!("{} {}", req_id, message);
    let (replies, timing) = perform_full_request(pool, &req_id, &message, None, None).await?;
    let rows = replies
        .iter()
        .map(|(node_alias, reply)| match reply {
            SingleReply::Reply(msg) => format!("{} {}", node_alias, msg),
            SingleReply::Failed(msg) => format!("{} failed: {}", node_alias, msg),
            SingleReply::Timeout() => format!("{} timeout", node_alias),
        })
        .collect::<Vec<String>>();
    Ok((rows.join("\n\n"), timing.unwrap()))
}

#[must_use = "requests do nothing unless polled"]
pub trait PoolRequest: std::fmt::Debug + Stream<Item = RequestEvent> + FusedStream + Unpin {
    fn clean_timeout(&self, node_alias: String) -> LedgerResult<()>;
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
    fn get_timing(&self) -> Option<TimingResult>;
    fn is_active(&self) -> bool;
    fn node_aliases(&self) -> Vec<String>;
    fn node_count(&self) -> usize;
    fn node_keys(&self) -> LedgerResult<Nodes>;
    fn send_to_all(&mut self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to_any(&mut self, count: usize, timeout: RequestTimeout) -> LedgerResult<Vec<String>>;
    fn send_to(
        &mut self,
        node_aliases: Vec<String>,
        timeout: RequestTimeout,
    ) -> LedgerResult<Vec<String>>;
}

struct PoolRequestImpl {
    handle: RequestHandle,
    events: Option<Receiver<RequestExtEvent>>,
    networker: Arc<RwLock<dyn Networker>>,
    node_aliases: Vec<String>,
    send_count: usize,
    state: RequestState,
    timing: RequestTiming,
}

impl PoolRequestImpl {
    unsafe_pinned!(events: Option<Receiver<RequestExtEvent>>);

    fn new(
        handle: RequestHandle,
        events: Receiver<RequestExtEvent>,
        networker: Arc<RwLock<dyn Networker>>,
        node_aliases: Vec<String>,
    ) -> Self {
        Self {
            handle,
            events: Some(events),
            networker,
            node_aliases,
            send_count: 0,
            state: RequestState::NotStarted,
            timing: RequestTiming::new(),
        }
    }

    fn trigger(&self, event: NetworkerEvent) -> LedgerResult<()> {
        self.networker
            .read()
            .map_err(|_| err_msg(LedgerErrorKind::InvalidState, "Error sending to networker"))?
            .send(event)
    }
}

impl PoolRequest for PoolRequestImpl {
    fn clean_timeout(&self, node_alias: String) -> LedgerResult<()> {
        self.trigger(NetworkerEvent::CleanTimeout(self.handle, node_alias))
    }

    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.trigger(NetworkerEvent::ExtendTimeout(
            self.handle,
            node_alias,
            timeout,
        ))
    }

    fn get_timing(&self) -> Option<TimingResult> {
        self.timing.get_result()
    }

    fn is_active(&self) -> bool {
        self.state == RequestState::Active
    }

    fn node_aliases(&self) -> Vec<String> {
        self.node_aliases.clone()
    }

    fn node_count(&self) -> usize {
        self.node_aliases.len()
    }

    fn node_keys(&self) -> LedgerResult<Nodes> {
        // FIXME - remove nodes that aren't present in node_aliases?
        Ok(self
            .networker
            .read()
            .map_err(|_| err_msg(LedgerErrorKind::InvalidState, "Error fetching node keys"))?
            .node_keys())
    }

    fn send_to_all(&mut self, timeout: RequestTimeout) -> LedgerResult<()> {
        let aliases = self.node_aliases();
        let count = aliases.len();
        self.trigger(NetworkerEvent::Dispatch(self.handle, aliases, timeout))?;
        self.send_count += count;
        Ok(())
    }

    fn send_to_any(&mut self, count: usize, timeout: RequestTimeout) -> LedgerResult<Vec<String>> {
        let aliases = self.node_aliases();
        let max = std::cmp::min(self.send_count + count, aliases.len());
        let min = std::cmp::min(self.send_count, max);
        trace!("send to any {} {} {:?}", min, max, aliases);
        let nodes = (min..max)
            .map(|idx| aliases[idx].clone())
            .collect::<Vec<String>>();
        if nodes.len() > 0 {
            self.trigger(NetworkerEvent::Dispatch(
                self.handle,
                nodes.clone(),
                timeout,
            ))?;
            self.send_count += nodes.len();
        }
        Ok(nodes)
    }

    fn send_to(
        &mut self,
        node_aliases: Vec<String>,
        timeout: RequestTimeout,
    ) -> LedgerResult<Vec<String>> {
        let aliases = self
            .node_aliases()
            .iter()
            .filter(|n| node_aliases.contains(n))
            .cloned()
            .collect::<Vec<String>>();
        if aliases.len() > 0 {
            self.trigger(NetworkerEvent::Dispatch(
                self.handle,
                aliases.clone(),
                timeout,
            ))?;
            self.send_count += aliases.len();
        }
        Ok(aliases)
    }
}

impl std::fmt::Debug for PoolRequestImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PoolRequest({}, state={})",
            self.handle.value(),
            self.state
        )
    }
}

impl Drop for PoolRequestImpl {
    fn drop(&mut self) {
        trace!("Finish dropped request: {}", self.handle);
        self.trigger(NetworkerEvent::FinishRequest(self.handle))
            .unwrap_or(()) // don't mind if the receiver disconnected
    }
}

impl Stream for PoolRequestImpl {
    type Item = RequestEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            trace!("poll_next");
            match self.state {
                RequestState::NotStarted => {
                    if let Some(events) = self.as_mut().events().as_pin_mut() {
                        match events.poll_next(cx) {
                            Poll::Ready(val) => {
                                if let Some(RequestExtEvent::Init()) = val {
                                    trace!("Request active {}", self.handle);
                                    self.state = RequestState::Active
                                } else {
                                    trace!("Request aborted {}", self.handle);
                                    // events.close(); ?
                                    self.as_mut().events().set(None);
                                    self.state = RequestState::Terminated
                                }
                            }
                            Poll::Pending => return Poll::Pending,
                        }
                    } else {
                        self.state = RequestState::Terminated
                    }
                }
                RequestState::Active => {
                    if let Some(events) = self.as_mut().events().as_pin_mut() {
                        match events.poll_next(cx) {
                            Poll::Ready(val) => match val {
                                Some(RequestExtEvent::Sent(alias, when)) => {
                                    trace!("{} was sent to {}", self.handle, alias);
                                    self.timing.sent(&alias, when)
                                }
                                Some(RequestExtEvent::Received(alias, message, meta, when)) => {
                                    trace!("{} response from {}", self.handle, alias);
                                    self.timing.received(&alias, when);
                                    return Poll::Ready(Some(RequestEvent::Received(
                                        alias, message, meta,
                                    )));
                                }
                                Some(RequestExtEvent::Timeout(alias)) => {
                                    trace!("{} timed out {}", self.handle, alias);
                                    return Poll::Ready(Some(RequestEvent::Timeout(alias)));
                                }
                                _ => {
                                    trace!("{} terminated", self.handle);
                                    // events.close(); ?
                                    self.as_mut().events().set(None);
                                    self.state = RequestState::Terminated
                                }
                            },
                            Poll::Pending => return Poll::Pending,
                        }
                    } else {
                        self.state = RequestState::Terminated
                    }
                }
                RequestState::Terminated => return Poll::Ready(None),
            }
        }
    }
}

impl FusedStream for PoolRequestImpl {
    fn is_terminated(&self) -> bool {
        self.state == RequestState::Terminated
    }
}

#[derive(Clone)]
pub struct Pool {
    config: PoolConfig,
    networker: Arc<RwLock<dyn Networker>>,
}

impl Pool {
    pub fn new(config: PoolConfig, networker: Arc<RwLock<dyn Networker>>) -> Self {
        Self { config, networker }
    }

    pub fn create_request<'a>(
        &'a self,
        req_id: String,
        req_json: String,
    ) -> LocalBoxFuture<'a, LedgerResult<Box<dyn PoolRequest>>> {
        let instance = self.networker.clone();
        lazy(move |_| {
            // FIXME - use unbounded channel? or require networker to dispatch from a queue
            let (tx, rx) = channel(10);
            trace!("{}", &req_json);
            let handle = RequestHandle::next();
            let inst_read = instance.read().unwrap();
            let node_aliases = inst_read.select_nodes();
            inst_read.send(NetworkerEvent::NewRequest(handle, req_id, req_json, tx))?;
            Ok(Box::new(PoolRequestImpl::new(
                handle,
                rx,
                instance.clone(),
                node_aliases,
            )) as Box<dyn PoolRequest>)
        })
        .boxed_local()
    }

    pub fn config(&self) -> PoolConfig {
        return self.config;
    }
}

/*
#[cfg(test)]
mod tests {
    // use crate::services::pool::events::MockUpdateHandler;
    use crate::services::pool::networker::MockNetworker;
    use crate::services::pool::request_handler::tests::MockRequestHandler;
    use crate::services::pool::types::{
        next_command_handle, next_pool_handle, Message, Reply, ReplyResultV1, ReplyTxnV1, ReplyV1,
        ResponseMetadata,
    };
    use crate::utils::test;
    use crate::utils::test::test_pool_create_poolfile;

    use super::*;

    const TEST_POOL_CONFIG: PoolConfig = PoolConfig::default();

    mod pool {
        use super::*;

        #[test]
        pub fn pool_new_works() {
            let _p: Pool<MockNetworker, MockRequestHandler> =
                Pool::new(next_pool_handle(), &TEST_POOL_CONFIG);
        }

        #[test]
        pub fn pool_get_id_works() {
            let id = next_pool_handle();
            let p: Pool<MockNetworker, MockRequestHandler> = Pool::new(id, &TEST_POOL_CONFIG);
            assert_eq!(id, p.get_id());
        }
    }

    mod pool_sm {
        use std::io::Write;

        use serde_json;

        use super::*;

        #[test]
        pub fn pool_wrapper_new_inactive_works() {
            let _p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
        }

        #[test]
        pub fn pool_wrapper_check_cache_works() {
            test::cleanup_storage("pool_wrapper_check_cache_works");

            _write_genesis_txns("pool_wrapper_check_cache_works");

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            assert_match!(PoolState::GettingCatchupTarget(_), p.state);

            test::cleanup_storage("pool_wrapper_check_cache_works");
        }

        #[test]
        pub fn pool_wrapper_check_cache_works_for_no_pool_created() {
            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            assert_match!(PoolState::Terminated(_), p.state);
        }

        #[test]
        pub fn pool_wrapper_terminated_close_works() {
            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::Close(cmd_id));
            assert_match!(PoolState::Closed(_), p.state);
        }

        #[test]
        pub fn pool_wrapper_terminated_refresh_works() {
            test::cleanup_pool("pool_wrapper_terminated_refresh_works");
            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));

            _write_genesis_txns("pool_wrapper_terminated_refresh_works");

            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::Refresh(cmd_id));
            assert_match!(PoolState::GettingCatchupTarget(_), p.state);
            test::cleanup_pool("pool_wrapper_terminated_refresh_works");
        }

        #[test]
        pub fn pool_wrapper_terminated_timeout_works() {
            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM {
                id: next_pool_handle(),
                config: TEST_POOL_CONFIG,
                state: PoolState::Terminated(TerminatedState {
                    networker: Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
                }),
            };

            let p = p.handle_event(PoolEvent::Timeout("".to_string(), "".to_string()));
            assert_match!(PoolState::Terminated(_), p.state);
            match p.state {
                PoolState::Terminated(state) => {
                    assert_eq!(state.networker.borrow().events.len(), 1);
                    let event = state.networker.borrow_mut().events.remove(0);
                    assert_match!(Some(NetworkerEvent::Timeout), event);
                }
                _ => assert!(false),
            }
        }

        #[test]
        pub fn pool_wrapper_close_works_from_inactive() {
            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::Close(cmd_id));
            assert_match!(PoolState::Closed(_), p.state);
        }

        #[test]
        pub fn pool_wrapper_close_works_from_getting_catchup_target() {
            test::cleanup_storage("pool_wrapper_close_works_from_getting_catchup_target");

            _write_genesis_txns("pool_wrapper_close_works_from_getting_catchup_target");

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::Close(cmd_id));
            assert_match!(PoolState::Closed(_), p.state);

            test::cleanup_storage("pool_wrapper_close_works_from_getting_catchup_target");
        }

        #[test]
        pub fn pool_wrapper_catchup_target_not_found_works() {
            test::cleanup_storage("pool_wrapper_catchup_target_not_found_works");

            _write_genesis_txns("pool_wrapper_catchup_target_not_found_works");

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::CatchupTargetNotFound(err_msg(
                LedgerErrorKind::PoolTimeout,
                "Pool timeout",
            )));
            assert_match!(PoolState::Terminated(_), p.state);

            test::cleanup_storage("pool_wrapper_catchup_target_not_found_works");
        }

        #[test]
        pub fn pool_wrapper_getting_catchup_target_synced_works() {
            test::cleanup_storage("pool_wrapper_getting_catchup_target_synced_works");

            _write_genesis_txns("pool_wrapper_getting_catchup_target_synced_works");

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            assert_match!(PoolState::Active(_), p.state);

            test::cleanup_storage("pool_wrapper_getting_catchup_target_synced_works");
        }

        /*
        FIXME changes protocol version
        #[test]
        pub fn pool_wrapper_getting_catchup_target_synced_works_for_node_state_error() {
            test::cleanup_storage(
                "pool_wrapper_getting_catchup_target_synced_works_for_node_state_error",
            );

            ProtocolVersion::set(2);
            _write_genesis_txns(
                "pool_wrapper_getting_catchup_target_synced_works_for_node_state_error",
            );

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            ProtocolVersion::set(1);
            let p = p.handle_event(PoolEvent::Synced(
                merkle_tree_factory::create(
                    "pool_wrapper_getting_catchup_target_synced_works_for_node_state_error",
                )
                .unwrap(),
            ));
            assert_match!(PoolState::Terminated(_), p.state);

            test::cleanup_storage(
                "pool_wrapper_getting_catchup_target_synced_works_for_node_state_error",
            );
        }
        */

        #[test]
        pub fn pool_wrapper_getting_catchup_target_catchup_target_found_works() {
            test::cleanup_storage("pool_wrapper_getting_catchup_target_catchup_target_found_works");

            _write_genesis_txns("pool_wrapper_getting_catchup_target_catchup_target_found_works");

            let mt = merkle_tree_factory::create(
                "pool_wrapper_getting_catchup_target_catchup_target_found_works",
            )
            .unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::CatchupTargetFound(
                mt.root_hash().to_vec(),
                mt.count,
                mt,
            ));
            assert_match!(PoolState::SyncCatchup(_), p.state);

            test::cleanup_storage("pool_wrapper_getting_catchup_target_catchup_target_found_works");
        }

        /*
        FIXME changed protocol version
        #[test]
        pub fn pool_wrapper_getting_catchup_target_catchup_target_found_works_for_node_state_error()
        {
            test::cleanup_storage("pool_wrapper_getting_catchup_target_catchup_target_found_works_for_node_state_error");

            ProtocolVersion::set(2);
            _write_genesis_txns("pool_wrapper_getting_catchup_target_catchup_target_found_works_for_node_state_error");

            let mt = merkle_tree_factory::create("pool_wrapper_getting_catchup_target_catchup_target_found_works_for_node_state_error").unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            ProtocolVersion::set(1);
            let p = p.handle_event(PoolEvent::CatchupTargetFound(
                mt.root_hash().to_vec(),
                mt.count,
                mt,
            ));
            assert_match!(PoolState::Terminated(_), p.state);

            test::cleanup_storage("pool_wrapper_getting_catchup_target_catchup_target_found_works_for_node_state_error");
        }
        */

        #[test]
        pub fn pool_wrapper_sync_catchup_close_works() {
            test::cleanup_storage("pool_wrapper_sync_catchup_close_works");

            _write_genesis_txns("pool_wrapper_sync_catchup_close_works");

            let mt = merkle_tree_factory::create("pool_wrapper_sync_catchup_close_works").unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::CatchupTargetFound(
                mt.root_hash().to_vec(),
                mt.count,
                mt,
            ));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::Close(cmd_id));
            assert_match!(PoolState::Closed(_), p.state);

            test::cleanup_storage("pool_wrapper_sync_catchup_close_works");
        }

        #[test]
        pub fn pool_wrapper_sync_catchup_synced_works() {
            test::cleanup_storage("pool_wrapper_sync_catchup_synced_works");

            _write_genesis_txns("pool_wrapper_sync_catchup_synced_works");

            let mt = merkle_tree_factory::create("pool_wrapper_sync_catchup_synced_works").unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::CatchupTargetFound(
                mt.root_hash().to_vec(),
                mt.count,
                mt,
            ));
            let p = p.handle_event(PoolEvent::Synced(
                merkle_tree_factory::create("pool_wrapper_sync_catchup_synced_works").unwrap(),
            ));
            assert_match!(PoolState::Active(_), p.state);

            test::cleanup_storage("pool_wrapper_sync_catchup_synced_works");
        }

        /*
        FIXME changes protocol version
        #[test]
        pub fn pool_wrapper_sync_catchup_synced_works_for_node_state_error() {
            test::cleanup_storage("pool_wrapper_sync_catchup_synced_works_for_node_state_error");

            ProtocolVersion::set(2);
            _write_genesis_txns("pool_wrapper_sync_catchup_synced_works_for_node_state_error");

            let mt = merkle_tree_factory::create(
                "pool_wrapper_sync_catchup_synced_works_for_node_state_error",
            )
            .unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::CatchupTargetFound(
                mt.root_hash().to_vec(),
                mt.count,
                mt,
            ));
            ProtocolVersion::set(1);
            let p = p.handle_event(PoolEvent::Synced(
                merkle_tree_factory::create(
                    "pool_wrapper_sync_catchup_synced_works_for_node_state_error",
                )
                .unwrap(),
            ));
            assert_match!(PoolState::Terminated(_), p.state);

            test::cleanup_storage("pool_wrapper_sync_catchup_synced_works_for_node_state_error");
        }
        */

        #[test]
        pub fn pool_wrapper_active_send_request_works() {
            test::cleanup_storage("pool_wrapper_active_send_request_works");

            _write_genesis_txns("pool_wrapper_active_send_request_works");

            let req = json!({
                "reqId": 1,
                "operation": {
                    "type": "1"
                }
            })
            .to_string();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.request_handlers.len(), 1);
                    assert!(state.request_handlers.contains_key("1"));
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_active_send_request_works");
        }

        #[test]
        pub fn pool_wrapper_active_send_request_works_for_no_req_id() {
            test::cleanup_storage("pool_wrapper_active_send_request_works_for_no_req_id");

            _write_genesis_txns("pool_wrapper_active_send_request_works_for_no_req_id");

            let req = json!({
                "operation": {
                    "type": "1"
                }
            })
            .to_string();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.request_handlers.len(), 0);
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_active_send_request_works_for_no_req_id");
        }

        #[test]
        pub fn pool_wrapper_active_node_reply_works() {
            test::cleanup_storage("pool_wrapper_active_node_reply_works");

            _write_genesis_txns("pool_wrapper_active_node_reply_works");

            let req = json!({
                "reqId": 1,
                "operation": {
                    "type": "1"
                }
            })
            .to_string();

            let rep = Message::Reply(Reply::ReplyV1(ReplyV1 {
                result: ReplyResultV1 {
                    txn: ReplyTxnV1 {
                        metadata: ResponseMetadata { req_id: 1 },
                    },
                },
            }));

            let rep = serde_json::to_string(&rep).unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            let p = p.handle_event(PoolEvent::NodeReply(rep, "node".to_string()));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.request_handlers.len(), 0);
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_active_node_reply_works");
        }

        #[test]
        pub fn pool_wrapper_sends_requests_to_two_nodes() {
            test::cleanup_storage("pool_wrapper_sends_requests_to_two_nodes");

            _write_genesis_txns("pool_wrapper_sends_requests_to_two_nodes");

            let req = json!({
                "reqId": 1,
                "operation": {
                    "type": "105"
                }
            })
            .to_string();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.networker.borrow().events.len(), 2);
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_sends_requests_to_two_nodes");
        }

        #[test]
        pub fn pool_wrapper_active_node_reply_works_for_no_request() {
            test::cleanup_storage("pool_wrapper_active_node_reply_works_for_no_request");

            _write_genesis_txns("pool_wrapper_active_node_reply_works_for_no_request");

            let req = json!({
                "reqId": 1,
                "operation": {
                    "type": "1"
                }
            })
            .to_string();

            let rep = Message::Reply(Reply::ReplyV1(ReplyV1 {
                result: ReplyResultV1 {
                    txn: ReplyTxnV1 {
                        metadata: ResponseMetadata { req_id: 2 },
                    },
                },
            }));

            let rep = serde_json::to_string(&rep).unwrap();

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            let p = p.handle_event(PoolEvent::NodeReply(rep, "node".to_string()));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.request_handlers.len(), 1);
                    assert!(state.request_handlers.contains_key("1"));
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_active_node_reply_works_for_no_request");
        }

        #[test]
        pub fn pool_wrapper_active_node_reply_works_for_invalid_reply() {
            test::cleanup_storage("pool_wrapper_active_node_reply_works_for_invalid_reply");

            _write_genesis_txns("pool_wrapper_active_node_reply_works_for_invalid_reply");

            let req = json!({
                "reqId": 1,
                "operation": {
                    "type": "1"
                }
            })
            .to_string();

            let rep = r#"{}"#;

            let p: PoolSM<MockNetworker, MockRequestHandler> = PoolSM::new(
                next_pool_handle(),
                &TEST_POOL_CONFIG,
                Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![]))),
            );
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::CheckCache(cmd_id));
            let p = p.handle_event(PoolEvent::Synced(MerkleTree::from_vec(vec![]).unwrap()));
            let cmd_id: CommandHandle = next_command_handle();
            let p = p.handle_event(PoolEvent::SendRequest(cmd_id, req, None, None));
            let p = p.handle_event(PoolEvent::NodeReply(rep.to_string(), "node".to_string()));
            assert_match!(PoolState::Active(_), p.state);
            match p.state {
                PoolState::Active(state) => {
                    assert_eq!(state.request_handlers.len(), 1);
                }
                _ => assert!(false),
            };

            test::cleanup_storage("pool_wrapper_active_node_reply_works_for_invalid_reply");
        }

        fn _write_genesis_txns(pool_name: &str) {
            let txns = test::gen_txns().join("\n");

            let mut f = test_pool_create_poolfile(pool_name);
            f.write(txns.as_bytes()).unwrap();
            f.flush().unwrap();
            f.sync_all().unwrap();
        }
    }

    mod other {
        use super::*;

        #[test]
        fn get_f_works() {
            test::cleanup_storage("get_f_works");

            assert_eq!(_get_f(0), 0);
            assert_eq!(_get_f(3), 0);
            assert_eq!(_get_f(4), 1);
            assert_eq!(_get_f(5), 1);
            assert_eq!(_get_f(6), 1);
            assert_eq!(_get_f(7), 2);
        }
    }
}
*/
