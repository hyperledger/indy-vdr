// use std::collections::BTreeMap;
use futures::executor::block_on;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::domain::did::{DidValue, DEFAULT_LIBINDY_DID};
use crate::domain::ledger::request::{get_request_id, Request};
use crate::domain::ledger::txn::{GetTxnOperation, LedgerType};

use super::events::PoolEvent;
use super::merkle_tree_factory::{build_tree, show_transactions};
use super::networker::{Networker, ZMQNetworker};
use super::requests::catchup::{perform_catchup_request, CatchupRequestResult};
use super::requests::single::{perform_single_request, SingleRequestResult};
use super::requests::status::{perform_status_request, StatusRequestResult};
use super::types::{CommandHandle, PoolConfig, DEFAULT_FRESHNESS_TIMEOUT, DEFAULT_GENERATOR};

use crate::utils::base58::ToBase58;
use crate::utils::error::prelude::*;

pub trait Pool {
    fn new(config: PoolConfig) -> Self
    where
        Self: Sized;
    fn connect(&mut self, json_transactions: Vec<String>) -> LedgerResult<CommandHandle>;
}

pub type ZMQPool = PoolImpl<ZMQNetworker>;

pub struct PoolImpl<T: Networker> {
    _pd: PhantomData<T>,
    evt_send: Sender<PoolEvent>,
    worker: Option<JoinHandle<()>>,
}

impl<T: Networker> Pool for PoolImpl<T> {
    fn new(config: PoolConfig) -> PoolImpl<T> {
        let (evt_send, evt_recv) = channel::<PoolEvent>();
        let mut result = Self {
            _pd: PhantomData,
            evt_send: evt_send.clone(),
            worker: None,
        };
        result.worker.replace(thread::spawn(move || {
            let mut pool_thread = PoolThread::<T>::new(config, evt_recv, evt_send);
            pool_thread.work();
            // FIXME send event when thread exits
            trace!("Pool thread exited")
        }));
        result
    }

    fn connect(&mut self, json_transactions: Vec<String>) -> LedgerResult<CommandHandle> {
        let cmd_id = CommandHandle::next();
        self.send(PoolEvent::Connect(cmd_id, json_transactions))?;
        Ok(cmd_id)
    }
}

impl<T: Networker> PoolImpl<T> {
    fn send(&self, event: PoolEvent) -> LedgerResult<()> {
        self.evt_send
            .send(event)
            .to_result(LedgerErrorKind::PoolTerminated, "Pool terminated")
    }
}

impl<T: Networker> Drop for PoolImpl<T> {
    fn drop(&mut self) {
        info!("Drop started");
        if let Err(_) = self.evt_send.send(PoolEvent::Exit()) {
            trace!("Pool thread already exited")
        }
        if let Some(worker) = self.worker.take() {
            info!("Drop pool worker");
            worker.join().unwrap();
        }
        info!("Drop finished");
    }
}

struct PoolStatusReq {
    cmd_id: Option<String>,
    req_id: String,
}

struct PoolThread<T: Networker> {
    config: PoolConfig,
    networker: Option<T>,
    evt_recv: Receiver<PoolEvent>,
    evt_send: Sender<PoolEvent>,
    txns: Vec<String>,
}

impl<T: Networker> PoolThread<T> {
    pub fn new(
        config: PoolConfig,
        evt_recv: Receiver<PoolEvent>,
        evt_send: Sender<PoolEvent>,
    ) -> Self {
        Self {
            config,
            networker: None,
            evt_recv,
            evt_send,
            txns: vec![],
        }
    }

    pub fn work(&mut self) {
        loop {
            let evt = match self.evt_recv.recv() {
                Ok(msg) => msg,
                _ => {
                    trace!("Pool thread exited");
                    return;
                }
            };
            if self.handle_event(evt) {
                break;
            }
        }
    }

    fn connect(&mut self, txns: Vec<String>) -> LedgerResult<()> {
        let networker = T::new(self.config, txns.clone(), vec![])?;
        block_on(
            // perform_connect(txns, &networker)
            perform_get_txn(LedgerType::DOMAIN, 2, &networker),
        )
        .unwrap();
        self.networker = Some(networker);
        Ok(())
    }

    fn handle_event(&mut self, event: PoolEvent) -> bool {
        match event {
            PoolEvent::Connect(_cmd_id, txns) => {
                // FIXME handle error, send update to listener
                self.connect(txns).unwrap();
            }
            PoolEvent::Exit() => {
                // networker(s) automatically dropped
                return true;
            } /*
              _ => trace!("Unhandled event {:?}", event),
              */
        };
        false
    }
}

async fn perform_connect<T: Networker>(txns: Vec<String>, networker: &T) -> LedgerResult<()> {
    let merkle_tree = build_tree(&txns)?;
    let result = perform_status_request(merkle_tree, networker).await?;
    trace!("Got status result: {:?}", &result);
    match result {
        StatusRequestResult::CatchupTargetFound(mt_root, mt_size, timing) => {
            trace!(
                "Catchup target found {} {} {:?}",
                mt_root.to_base58(),
                mt_size,
                timing
            );
            perform_catchup(txns, networker, mt_root, mt_size).await
        }
        StatusRequestResult::CatchupTargetNotFound(err, timing) => {
            trace!("Catchup target not found {:?}", timing);
            Err(err)
        }
        StatusRequestResult::Synced(timing) => {
            trace!("Synced! {:?}", timing);
            Ok(())
        }
    }
}

async fn perform_catchup<T: Networker>(
    txns: Vec<String>,
    networker: &T,
    mt_root: Vec<u8>,
    mt_size: usize,
) -> LedgerResult<()> {
    let merkle_tree = build_tree(&txns)?;
    let catchup_result = perform_catchup_request(merkle_tree, mt_root, mt_size, networker).await?;
    trace!("Got catchup result: {:?}", &catchup_result);
    match catchup_result {
        CatchupRequestResult::Synced(txns, timing) => {
            trace!("Catchup synced! {:?}", timing);
            let txns = show_transactions(&txns, networker.protocol_version())?;
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

fn build_get_txn_request(
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

async fn perform_get_txn<T: Networker>(
    ledger_type: LedgerType,
    seq_no: i32,
    networker: &T,
) -> LedgerResult<String> {
    let (req_id, message) = build_get_txn_request(
        ledger_type.to_id(),
        seq_no,
        None,
        Some(networker.protocol_version().to_id()),
    )?;
    trace!("{} {}", req_id, message);
    let result = perform_single_request(
        &req_id,
        &message,
        2,
        None,
        (None, None),
        DEFAULT_FRESHNESS_TIMEOUT, // FIXME - take from PoolConfig
        networker,
    )
    .await?;
    match result {
        SingleRequestResult::Response(message, timing) => {
            trace!("Got request response {} {:?}", &message, timing);
            Ok(message)
        }
        SingleRequestResult::NoConsensus(timing) => {
            trace!("No consensus {:?}", timing);
            Err(err_msg(LedgerErrorKind::PoolTimeout, "No consensus"))
        }
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
