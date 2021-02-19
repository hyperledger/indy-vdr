use std::rc::Rc;
use std::sync::{Arc, Mutex};

use futures_channel::mpsc::UnboundedSender;

use crate::common::error::prelude::*;
use crate::config::types::PoolConfig;

use super::requests::RequestExtEvent;
use super::types::{self, RequestHandle, Verifiers};

mod zmq;
pub use self::zmq::{ZMQNetworker, ZMQNetworkerFactory};

/// Events used to drive a `Networker` instance
#[derive(Debug)]
pub enum NetworkerEvent {
    FinishRequest(RequestHandle),
    NewRequest(
        RequestHandle,
        String, // subscribe to ID
        String, // message body
        UnboundedSender<RequestExtEvent>,
    ),
    Dispatch(
        RequestHandle,
        Vec<String>, // node aliases
        i64,         // timeout
    ),
    CleanTimeout(
        RequestHandle,
        String, // node alias
    ),
    ExtendTimeout(
        RequestHandle,
        String, // node alias
        i64,    // timeout
    ),
}

/// A simple trait implemented by all networker types
pub trait Networker {
    fn send(&self, event: NetworkerEvent) -> VdrResult<()>;
}

/// A factory for `Networker` instances
pub trait NetworkerFactory {
    type Output: Networker;
    fn make_networker(&self, config: PoolConfig, verifiers: &Verifiers) -> VdrResult<Self::Output>;
}

/// A `Networker` instance which can be cloned and used within one thread
pub type LocalNetworker = Rc<dyn Networker + 'static>;

/// A derived `NetworkerFactory` producing thread-local cloneable instances
pub struct MakeLocal<T: NetworkerFactory>(pub T);

impl<T> NetworkerFactory for MakeLocal<T>
where
    T: NetworkerFactory,
    T::Output: Networker + 'static,
{
    type Output = LocalNetworker;
    fn make_networker(&self, config: PoolConfig, verifiers: &Verifiers) -> VdrResult<Self::Output> {
        Ok(Rc::new(self.0.make_networker(config, verifiers)?))
    }
}

impl<T> Networker for T
where
    T: AsRef<dyn Networker>,
{
    fn send(&self, event: NetworkerEvent) -> VdrResult<()> {
        self.as_ref().send(event)
    }
}

/// A `Networker` instance which can be cloned and used across multiple threads
pub type SharedNetworker = Arc<Mutex<dyn Networker + Send + 'static>>;

/// A derived `NetworkerFactory` producing shareable instances
pub struct MakeShared<T: NetworkerFactory>(pub T);

impl<T> NetworkerFactory for MakeShared<T>
where
    T: NetworkerFactory,
    T::Output: Networker + Send + 'static,
{
    type Output = SharedNetworker;
    fn make_networker(&self, config: PoolConfig, verifiers: &Verifiers) -> VdrResult<Self::Output> {
        Ok(Arc::new(Mutex::new(
            self.0.make_networker(config, verifiers)?,
        )))
    }
}

impl Networker for Arc<Mutex<dyn Networker + Send>> {
    fn send(&self, event: NetworkerEvent) -> VdrResult<()> {
        self.lock()
            .map_err(|_| {
                err_msg(
                    VdrErrorKind::Unexpected,
                    "Error acquiring networker, mutex poisoned",
                )
            })?
            .send(event)
    }
}

/*
#[cfg(test)]
pub struct MockNetworker {
    pub id: NetworkerHandle,
    pub events: Vec<Option<NetworkerEvent>>,
}

#[cfg(test)]
impl Networker for MockNetworker {
    fn new(
        _config: PoolConfig,
        _transactions: Vec<String>,
        _preferred_nodes: Vec<String>,
    ) -> VdrResult<Self> {
        Ok(Self {
            id: NetworkerHandle::next(),
            events: Vec::new(),
        })
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

    /*fn add_request(&mut self, _request: PoolRequest) -> VdrResult<()> {
        unimplemented!()
    }*/

    fn create_request<'a>(
        &'a mut self,
        _message: &Message,
    ) -> LocalBoxFuture<'a, VdrResult<RefCell<Box<dyn NetworkerRequest>>>> {
        unimplemented!()
    }
}
*/

/*
#[cfg(test)]
pub mod networker_tests {
    use std;
    use std::thread;

    use super::super::types::{
        DEFAULT_ACK_TIMEOUT, DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT,
        DEFAULT_REPLY_TIMEOUT,
    };
    use crate::services::pool::tests::nodes_emulator;
    use crate::utils::base58::FromBase58;
    use crate::utils::crypto;

    use super::*;

    const REQ_ID: &str = "1";
    const MESSAGE: &str = "msg";
    const NODE_NAME: &str = "n1";

    pub fn _remote_node(txn: &NodeTransactionV1) -> RemoteNode {
        let vk = crypto::PublicKey::from_bytes(&txn.txn.data.dest.as_str().from_base58().unwrap())
            .unwrap();
        RemoteNode {
            public_key: crypto::vk_to_curve25519(vk).unwrap(),
            zaddr: format!(
                "tcp://{}:{}",
                txn.txn.data.data.client_ip.clone().unwrap(),
                txn.txn.data.data.client_port.clone().unwrap()
            ),
            name: txn.txn.data.data.alias.clone(),
            is_blacklisted: false,
        }
    }

    #[cfg(test)]
    mod networker {
        use std::ops::Sub;

        use super::*;

        #[test]
        pub fn networker_new_works() {
            ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
        }

        #[test]
        pub fn networker_process_event_works() {
            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(None);
        }

        #[test]
        fn networker_process_update_node_state_event_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            assert_eq!(0, networker.nodes.len());

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            assert_eq!(1, networker.nodes.len());
        }

        #[test]
        fn networker_process_send_request_event_works() {
            let mut txn = nodes_emulator::node();
            let handle = nodes_emulator::start(&mut txn);
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            assert!(networker.pool_connections.is_empty());
            assert!(networker.req_id_mappings.is_empty());

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            assert_eq!(1, networker.pool_connections.len());
            assert_eq!(1, networker.req_id_mappings.len());
            assert!(networker.req_id_mappings.contains_key(REQ_ID));

            assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
            assert!(nodes_emulator::next(&handle).is_none());
        }

        #[test]
        fn networker_process_send_all_request_event_works() {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn_1, rn_2])));
            networker.process_event(Some(NetworkerEvent::SendAllRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
                None,
            )));

            for handle in vec![handle_1, handle_2] {
                assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
                assert!(nodes_emulator::next(&handle).is_none());
            }
        }

        #[test]
        fn networker_process_send_all_request_event_works_for_2_requests_and_different_nodes_order()
        {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let send_cnt = 2;

            let mut networker = ZMQNetworker::new(
                DEFAULT_CONN_ACTIVE_TIMEOUT,
                DEFAULT_CONN_REQ_LIMIT,
                vec!["n2".to_string(), "n1".to_string()],
            );

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn_1, rn_2])));

            for i in 0..send_cnt {
                networker.process_event(Some(NetworkerEvent::SendAllRequest(
                    MESSAGE.to_string(),
                    i.to_string(),
                    DEFAULT_ACK_TIMEOUT,
                    None,
                )));
                assert_eq!(1, networker.pool_connections.len());
            }

            for handle in vec![handle_1, handle_2] {
                let mut messages = Vec::new();
                for _ in 0..send_cnt {
                    messages.push(nodes_emulator::next(&handle).unwrap());
                }
                assert!(nodes_emulator::next(&handle).is_none());
                assert_eq!(vec![MESSAGE.to_string(); send_cnt], messages);
            }
        }

        #[test]
        fn networker_process_send_all_request_event_works_for_list_nodes() {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn_1, rn_2])));
            networker.process_event(Some(NetworkerEvent::SendAllRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
                Some(vec![NODE_NAME.to_string()]),
            )));

            assert_eq!(
                MESSAGE.to_string(),
                nodes_emulator::next(&handle_1).unwrap()
            );
            assert!(nodes_emulator::next(&handle_1).is_none());

            assert!(nodes_emulator::next(&handle_2).is_none());
        }

        #[test]
        fn networker_process_send_six_request_event_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            for i in 0..5 {
                networker.process_event(Some(NetworkerEvent::SendOneRequest(
                    MESSAGE.to_string(),
                    i.to_string(),
                    DEFAULT_ACK_TIMEOUT,
                )));
                assert_eq!(1, networker.pool_connections.len());
            }

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                "6".to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));
            assert_eq!(2, networker.pool_connections.len());

            let mut pc_iter = networker.pool_connections.values();
            let first_pc = pc_iter.next().unwrap();
            let second_pc = pc_iter.next().unwrap();
            assert_ne!(first_pc.key_pair.public_key, second_pc.key_pair.public_key);
        }

        #[test]
        fn networker_process_send_six_request_event_with_timeout_cleaning_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            for i in 0..5 {
                networker.process_event(Some(NetworkerEvent::SendOneRequest(
                    MESSAGE.to_string(),
                    i.to_string(),
                    DEFAULT_ACK_TIMEOUT,
                )));
            }
            assert_eq!(1, networker.pool_connections.len());

            for i in 0..5 {
                networker.process_event(Some(NetworkerEvent::CleanTimeout(i.to_string(), None)));
            }
            assert_eq!(1, networker.pool_connections.len());

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                "6".to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));
            assert_eq!(2, networker.pool_connections.len());
        }

        #[test]
        fn networker_process_extend_timeout_event_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));
            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            thread::sleep(std::time::Duration::from_secs(1));

            let (_, timeout) = networker.get_timeout();

            networker.process_event(Some(NetworkerEvent::ExtendTimeout(
                REQ_ID.to_string(),
                txn.txn.data.data.alias,
                DEFAULT_REPLY_TIMEOUT,
            )));

            let (_, timeout_2) = networker.get_timeout();

            assert!(timeout_2 > timeout);
        }

        // Roll back connection creation time on 5 seconds ago instead of sleeping
        fn _roll_back_timeout(networker: &mut ZMQNetworker) {
            let conn_id: u32 = networker
                .pool_connections
                .keys()
                .cloned()
                .collect::<Vec<u32>>()[0];
            let conn: &mut PoolConnection = networker.pool_connections.get_mut(&conn_id).unwrap();
            conn.time_created = time::now().sub(Duration::seconds(5));
        }

        #[test]
        fn networker_process_timeout_event_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);
            let conn = PoolConnection::new(vec![rn.clone()], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            networker.pool_connections.insert(1, conn);

            _roll_back_timeout(&mut networker);

            networker.process_event(Some(NetworkerEvent::Timeout));

            assert!(networker.pool_connections.is_empty());
        }

        #[test]
        fn networker_process_clean_timeout_event_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));
            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            _roll_back_timeout(&mut networker);

            networker.process_event(Some(NetworkerEvent::CleanTimeout(
                REQ_ID.to_string(),
                Some(txn.txn.data.data.alias),
            )));

            assert!(networker.pool_connections.is_empty());
        }

        #[test]
        fn networker_process_second_request_after_cleaning_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));
            networker.process_event(Some(NetworkerEvent::CleanTimeout(REQ_ID.to_string(), None)));

            assert_eq!(1, networker.pool_connections.len());

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                "2".to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            assert_eq!(1, networker.pool_connections.len());
        }

        #[test]
        fn networker_process_second_request_after_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);
            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            assert_eq!(1, networker.pool_connections.len());

            _roll_back_timeout(&mut networker);

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                "2".to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            assert_eq!(2, networker.pool_connections.len());
        }

        #[test]
        fn networker_get_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut networker =
                ZMQNetworker::new(DEFAULT_CONN_ACTIVE_TIMEOUT, DEFAULT_CONN_REQ_LIMIT, vec![]);

            networker.process_event(Some(NetworkerEvent::NodesStateUpdated(vec![rn])));

            let (_, timeout) = networker.get_timeout();

            assert_eq!(::std::i64::MAX, timeout);

            networker.process_event(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));

            let (_, timeout) = networker.get_timeout();

            assert_ne!(::std::i64::MAX, timeout);
        }
    }

    #[cfg(test)]
    mod remote_node {
        use super::*;

        #[test]
        fn remote_node_connect() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let _socket = rn
                .connect(&zmq::Context::new(), &zmq::CurveKeyPair::new().unwrap())
                .unwrap();
        }

        #[test]
        fn remote_node_connect_works_for_invalid_address() {
            let txn = nodes_emulator::node();
            let mut rn = _remote_node(&txn);
            rn.zaddr = "invalid_address".to_string();

            let res = rn.connect(&zmq::Context::new(), &zmq::CurveKeyPair::new().unwrap());
            assert_kind!(VdrErrorKind::IOError, res);
        }
    }

    #[cfg(test)]
    mod pool_connection {
        use std::ops::Sub;

        use super::*;

        #[test]
        fn pool_connection_new_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);
        }

        #[test]
        fn pool_connection_new_shuffle() {
            let mut txn = nodes_emulator::node();

            let mut exp_names: Vec<String> = Vec::new();
            let mut nodes: Vec<RemoteNode> = Vec::new();

            for i in 0..100 {
                txn.txn.data.data.alias = format!("Node{}", i);
                exp_names.push(txn.txn.data.data.alias.clone());
                nodes.push(_remote_node(&txn));
            }

            let pc = PoolConnection::new(nodes, DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let act_names: Vec<String> = pc.nodes.iter().map(|n| n.name.to_string()).collect();

            assert_ne!(exp_names, act_names);
        }

        #[test]
        fn pool_connection_new_works_for_preordered_nodes() {
            let mut txn = nodes_emulator::node();

            txn.txn.data.data.alias = "Node1".to_string();
            let rn_1 = _remote_node(&txn);

            txn.txn.data.data.alias = "Node2".to_string();
            let rn_2 = _remote_node(&txn);

            txn.txn.data.data.alias = "Node3".to_string();
            let rn_3 = _remote_node(&txn);

            txn.txn.data.data.alias = "Node4".to_string();
            let rn_4 = _remote_node(&txn);

            txn.txn.data.data.alias = "Node5".to_string();
            let rn_5 = _remote_node(&txn);

            let pc = PoolConnection::new(
                vec![
                    rn_1.clone(),
                    rn_2.clone(),
                    rn_3.clone(),
                    rn_4.clone(),
                    rn_5.clone(),
                ],
                DEFAULT_CONN_ACTIVE_TIMEOUT,
                vec![rn_2.name.clone(), rn_1.name.clone(), rn_5.name.clone()],
            );

            assert_eq!(rn_2.name, pc.nodes[0].name);
            assert_eq!(rn_1.name, pc.nodes[1].name);
            assert_eq!(rn_5.name, pc.nodes[2].name);
        }

        #[test]
        fn pool_connection_is_active_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            assert!(conn.is_active());

            conn.time_created = time::now().sub(Duration::seconds(DEFAULT_CONN_ACTIVE_TIMEOUT));

            assert!(!conn.is_active());
        }

        #[test]
        fn pool_connection_has_active_requests_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            assert!(!conn.has_active_requests());

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            assert!(conn.has_active_requests());
        }

        #[test]
        fn pool_connection_get_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let ((req_id, node_alias), timeout) = conn.get_timeout();
            assert_eq!(req_id, "".to_string());
            assert_eq!(node_alias, "".to_string());
            assert!(DEFAULT_CONN_ACTIVE_TIMEOUT * 1000 - 10 <= timeout);
            assert!(DEFAULT_CONN_ACTIVE_TIMEOUT * 1000 >= timeout);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            let (id, timeout) = conn.get_timeout();
            assert_eq!((REQ_ID.to_string(), NODE_NAME.to_string()), id);
            assert!(DEFAULT_ACK_TIMEOUT * 1000 - 10 <= timeout);
            assert!(DEFAULT_ACK_TIMEOUT * 1000 >= timeout);
        }

        #[test]
        fn pool_connection_extend_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            thread::sleep(std::time::Duration::from_secs(1));

            let ((msg, name), timeout) = conn.get_timeout();

            conn.extend_timeout(&msg, &name, DEFAULT_REPLY_TIMEOUT);

            let ((_, _), timeout_2) = conn.get_timeout();

            assert!(timeout_2 > timeout);
        }

        #[test]
        fn pool_connection_clean_timeout_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            assert!(conn.has_active_requests());

            conn.clean_timeout(REQ_ID, Some(NODE_NAME.to_string()));

            assert!(!conn.has_active_requests());
        }

        #[test]
        fn pool_connection_get_socket_works() {
            let txn = nodes_emulator::node();
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let _socket = conn._get_socket(0).unwrap();
        }

        #[test]
        fn pool_connection_get_socket_works_for_invalid_node_address() {
            let txn = nodes_emulator::node();
            let mut rn = _remote_node(&txn);
            rn.zaddr = "invalid_address".to_string();

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let res = conn._get_socket(0);
            assert_kind!(VdrErrorKind::IOError, res);
        }

        #[test]
        fn pool_connection_send_request_one_node_works() {
            let mut txn = nodes_emulator::node();
            let handle = nodes_emulator::start(&mut txn);
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();
            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                "msg2".to_string(),
                "12".to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
            assert_eq!("msg2".to_string(), nodes_emulator::next(&handle).unwrap());
            assert!(nodes_emulator::next(&handle).is_none());
        }

        #[test]
        fn pool_connection_send_request_one_node_works_for_two_active_nodes() {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let mut conn = PoolConnection::new(
                vec![rn_1, rn_2],
                DEFAULT_CONN_ACTIVE_TIMEOUT,
                vec!["n1".to_string(), "n2".to_string()],
            );

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            assert_eq!(
                MESSAGE.to_string(),
                nodes_emulator::next(&handle_1).unwrap()
            );
            assert!(nodes_emulator::next(&handle_1).is_none());

            assert!(nodes_emulator::next(&handle_2).is_none());
        }

        #[test]
        fn pool_connection_send_request_all_nodes_works() {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let mut conn =
                PoolConnection::new(vec![rn_1, rn_2], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendAllRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
                None,
            )))
            .unwrap();

            for handle in vec![handle_1, handle_2] {
                assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
                assert!(nodes_emulator::next(&handle).is_none());
            }
        }

        #[test]
        fn pool_connection_resend_works() {
            let mut txn = nodes_emulator::node();
            let handle = nodes_emulator::start(&mut txn);
            let rn = _remote_node(&txn);

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            conn.send_request(Some(NetworkerEvent::Resend(
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
            assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
            assert!(nodes_emulator::next(&handle).is_none());
        }

        #[test]
        fn pool_connection_resend_works_for_two_nodes() {
            let mut txn_1 = nodes_emulator::node();
            let handle_1 = nodes_emulator::start(&mut txn_1);
            let rn_1 = _remote_node(&txn_1);

            let mut txn_2 = nodes_emulator::node_2();
            let handle_2 = nodes_emulator::start(&mut txn_2);
            let rn_2 = _remote_node(&txn_2);

            let mut conn =
                PoolConnection::new(vec![rn_1, rn_2], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            conn.send_request(Some(NetworkerEvent::Resend(
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )))
            .unwrap();

            for handle in vec![handle_1, handle_2] {
                assert_eq!(MESSAGE.to_string(), nodes_emulator::next(&handle).unwrap());
                assert!(nodes_emulator::next(&handle).is_none());
            }
        }

        #[test]
        fn pool_connection_send_works_for_invalid_node() {
            let txn = nodes_emulator::node();
            let mut rn = _remote_node(&txn);
            rn.zaddr = "invalid_address".to_string();

            let mut conn = PoolConnection::new(vec![rn], DEFAULT_CONN_ACTIVE_TIMEOUT, vec![]);

            let res = conn.send_request(Some(NetworkerEvent::SendOneRequest(
                MESSAGE.to_string(),
                REQ_ID.to_string(),
                DEFAULT_ACK_TIMEOUT,
            )));
            assert_kind!(VdrErrorKind::IOError, res);
        }
    }
}
*/
