#![warn(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::pin::Pin;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use futures::channel::mpsc::{Receiver, Sender};
use futures::future::LocalBoxFuture;
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};

use pin_utils::unsafe_pinned;

use crate::utils::error::prelude::*;

use super::types::{Message, Nodes, PoolConfig};

mod zmq;
pub use self::zmq::ZMQNetworker;

new_handle_type!(RequestHandle, RQ_COUNTER);

new_handle_type!(NetworkerHandle, NH_COUNTER);

#[derive(Debug)]
enum NetworkerEvent {
    CancelRequest(RequestHandle),
    NewRequest(
        RequestHandle,
        String, // subscribe to ID
        String, // message body
        Sender<RequestExtEvent>,
    ),
    Dispatch(RequestHandle, RequestDispatchTarget, RequestTimeout),
    ExtendTimeout(
        RequestHandle,
        String, // node alias
        RequestTimeout,
    ),
}

#[derive(Debug)]
pub enum RequestEvent {
    Received(
        String, // node alias
        Message,
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug)]
enum RequestExtEvent {
    Init(Nodes),
    Sent(
        String,     // node alias
        SystemTime, // send time
        usize,      // send index
    ),
    Received(
        String, // node alias
        Message,
        SystemTime, // received time
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug, PartialEq, Eq)]
enum RequestDispatchTarget {
    AllNodes,
    AnyNode,
    SelectNode(String),
}

#[derive(Debug, PartialEq, Eq)]
enum RequestState {
    NotStarted,
    Active,
    Terminated,
}

impl std::fmt::Display for RequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::NotStarted => "NotStarted",
            Self::Active => "Active",
            Self::Terminated => "Terminated",
        };
        f.write_str(state)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestTimeout {
    Default,
    Ack,
    Seconds(i64),
}

impl RequestTimeout {
    pub fn expand(&self, config: &PoolConfig) -> i64 {
        match self {
            Self::Default => config.reply_timeout,
            Self::Ack => config.ack_timeout,
            Self::Seconds(n) => *n,
        }
    }
}

#[derive(Debug)]
struct RequestTiming {
    replies: HashMap<String, (SystemTime, f32)>,
}

impl RequestTiming {
    fn new() -> Self {
        Self {
            replies: HashMap::new(),
        }
    }

    fn sent(&mut self, node_alias: &str, send_time: SystemTime) {
        self.replies
            .insert(node_alias.to_owned(), (send_time, -1.0));
    }

    fn received(&mut self, node_alias: &str, recv_time: SystemTime) {
        self.replies.get_mut(node_alias).map(|node| {
            let duration = recv_time
                .duration_since(node.0)
                .unwrap_or(Duration::new(0, 0))
                .as_secs_f32();
            node.1 = duration;
        });
    }

    fn get_result(&self) -> Option<HashMap<String, f32>> {
        Some(HashMap::from_iter(
            self.replies.iter().map(|(k, (_, v))| (k.clone(), *v)),
        ))
    }
}

trait NetworkerSender: Sized {
    fn send(&self, event: NetworkerEvent) -> LedgerResult<()>;
}

#[must_use = "requests do nothing unless polled"]
pub trait NetworkerRequest:
    std::fmt::Debug + Stream<Item = RequestEvent> + FusedStream + Unpin
{
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
    fn get_nodes(&self) -> Option<Nodes>;
    fn get_timing(&self) -> Option<HashMap<String, f32>>;
    fn is_active(&self) -> bool;
    fn send_to_all(&self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to_any(&self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
}

struct NetworkerRequestImpl<T: NetworkerSender> {
    handle: RequestHandle,
    events: Option<Receiver<RequestExtEvent>>,
    nodes: Option<Nodes>,
    sender: Rc<RefCell<T>>,
    state: RequestState,
    timing: RequestTiming,
}

impl<T: NetworkerSender> NetworkerRequestImpl<T> {
    unsafe_pinned!(events: Option<Receiver<RequestExtEvent>>);

    fn new(
        handle: RequestHandle,
        events: Receiver<RequestExtEvent>,
        sender: Rc<RefCell<T>>,
    ) -> Self {
        Self {
            handle,
            events: Some(events),
            nodes: None,
            sender,
            state: RequestState::NotStarted,
            timing: RequestTiming::new(),
        }
    }
}

impl<T: NetworkerSender> NetworkerRequest for NetworkerRequestImpl<T> {
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.sender.borrow_mut().send(NetworkerEvent::ExtendTimeout(
            self.handle,
            node_alias,
            timeout,
        ))
    }

    fn get_nodes(&self) -> Option<Nodes> {
        return self.nodes.clone();
    }

    fn get_timing(&self) -> Option<HashMap<String, f32>> {
        self.timing.get_result()
    }

    fn is_active(&self) -> bool {
        self.state == RequestState::Active
    }

    fn send_to_all(&self, timeout: RequestTimeout) -> LedgerResult<()> {
        self.sender.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            RequestDispatchTarget::AllNodes,
            timeout,
        ))
    }

    fn send_to_any(&self, timeout: RequestTimeout) -> LedgerResult<()> {
        self.sender.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            RequestDispatchTarget::AnyNode,
            timeout,
        ))
    }

    fn send_to(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.sender.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            RequestDispatchTarget::SelectNode(node_alias),
            timeout,
        ))
    }
}

impl<T: NetworkerSender> std::fmt::Debug for NetworkerRequestImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NetworkerRequest({}, state={})",
            self.handle.value(),
            self.state
        )
    }
}

/*
impl Drop for ZMQRequest {
    fn drop(&mut self) {
        self.events.close();
        if !self.events.is_terminated() {
            trace!("draining zmq request");
            loop {
                if let Err(_) = self.events.try_next() {
                    break;
                }
            }
        }
    }
}
*/

impl<T: NetworkerSender> Stream for NetworkerRequestImpl<T> {
    type Item = RequestEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            trace!("poll_next");
            match self.state {
                RequestState::NotStarted => {
                    if let Some(events) = self.as_mut().events().as_pin_mut() {
                        match events.poll_next(cx) {
                            Poll::Ready(val) => {
                                if let Some(RequestExtEvent::Init(nodes)) = val {
                                    trace!("got init!");
                                    self.nodes.replace(nodes);
                                    self.state = RequestState::Active;
                                } else {
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
                                Some(RequestExtEvent::Sent(alias, when, _index)) => {
                                    self.timing.sent(&alias, when)
                                }
                                Some(RequestExtEvent::Received(alias, message, when)) => {
                                    self.timing.received(&alias, when);
                                    return Poll::Ready(Some(RequestEvent::Received(
                                        alias, message,
                                    )));
                                }
                                Some(RequestExtEvent::Timeout(alias)) => {
                                    return Poll::Ready(Some(RequestEvent::Timeout(alias)));
                                }
                                _ => {
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

impl<T: NetworkerSender> FusedStream for NetworkerRequestImpl<T> {
    fn is_terminated(&self) -> bool {
        self.state == RequestState::Terminated
    }
}

pub trait Networker: Sized {
    fn new(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
    ) -> LedgerResult<Self>;
    fn get_id(&self) -> NetworkerHandle;
    //fn add_request(&mut self, request: PoolRequest) -> LedgerResult<()>;
    fn create_request<'a>(
        &'a mut self,
        message: &Message,
    ) -> LocalBoxFuture<'a, LedgerResult<Box<dyn NetworkerRequest>>>;
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
    ) -> LedgerResult<Self> {
        Ok(Self {
            id: NetworkerHandle::next(),
            events: Vec::new(),
        })
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

    /*fn add_request(&mut self, _request: PoolRequest) -> LedgerResult<()> {
        unimplemented!()
    }*/

    fn create_request<'a>(
        &'a mut self,
        _message: &Message,
    ) -> LocalBoxFuture<'a, LedgerResult<RefCell<Box<dyn NetworkerRequest>>>> {
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
            assert_kind!(LedgerErrorKind::IOError, res);
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
            assert_kind!(LedgerErrorKind::IOError, res);
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
            assert_kind!(LedgerErrorKind::IOError, res);
        }
    }
}
*/
