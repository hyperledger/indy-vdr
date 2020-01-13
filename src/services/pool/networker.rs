use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::{lazy, FutureExt, LocalBoxFuture};
#[macro_use]
use futures::stream::{self, BoxStream, StreamExt};

use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::FromBase58;
use crate::utils::crypto;
use crate::utils::error::prelude::*;

use super::merkle_tree_factory::build_node_state_from_json;
use super::request_handler::{HandlerEvent, PoolRequest, PoolRequestTarget};
use super::types::{Message, Nodes, PoolConfig};

use super::zmq::PollItem;
use super::zmq::Socket as ZSocket;

use base64;
use ursa::bls::VerKey as BlsVerKey;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RemoteNode {
    pub name: String,
    pub public_key: Vec<u8>,
    pub zaddr: String,
    pub is_blacklisted: bool,
}

new_handle_type!(RequestHandle, RQ_COUNTER);

new_handle_type!(ZMQConnectionHandle, PHC_COUNTER);

new_handle_type!(NetworkerHandle, NH_COUNTER);

#[derive(Debug)]
pub enum NetworkerEvent {
    CancelRequest(RequestHandle),
    NewRequest(
        RequestHandle,
        String, // subscribe to ID
        String, // message body
        Sender<HandlerEvent>,
    ),
    Dispatch(RequestHandle, DispatchTarget, RequestTimeout),
    ExtendTimeout(
        RequestHandle,
        String, // node alias
        RequestTimeout,
    ),
}

#[derive(Debug)]
enum NodeEvent {
    Reply(
        String, // req id
        Message,
        String,     // node alias
        SystemTime, // received time
    ),
    Timeout(
        String, // req id
        String, // node alias
    ),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DispatchTarget {
    AllNodes,
    AnyNode,
    SelectNode(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestTimeout {
    Default,
    Ack,
    Seconds(i64),
}

impl RequestTimeout {
    fn expand(&self, config: &PoolConfig) -> i64 {
        match self {
            Self::Default => config.reply_timeout,
            Self::Ack => config.ack_timeout,
            Self::Seconds(n) => *n,
        }
    }
}

enum PollResult {
    Default,
    Events(Vec<(ZMQConnectionHandle, NodeEvent)>),
    NoSockets,
    Exit,
}

struct PendingRequest {
    conn_id: ZMQConnectionHandle,
    send_index: usize,
    sender: Sender<HandlerEvent>,
    sub_id: String,
    body: String,
}

pub trait Request: std::fmt::Debug {
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
    fn get_nodes(&self) -> Option<Nodes>;
    fn get_stream<'a>(&'a mut self) -> BoxStream<'a, HandlerEvent>;
    fn get_timing(&self) -> Option<HashMap<String, f32>>;
    fn is_active(&self) -> bool;
    fn send_to_all(&self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to_any(&self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
}

pub struct ZMQRequest {
    active: bool,
    handle: RequestHandle,
    events: Receiver<HandlerEvent>,
    instance: Rc<RefCell<ZMQNetworkerInstance>>,
}

impl ZMQRequest {
    fn new(
        handle: RequestHandle,
        events: Receiver<HandlerEvent>,
        instance: Rc<RefCell<ZMQNetworkerInstance>>,
    ) -> Self {
        Self {
            active: false,
            handle,
            events,
            instance,
        }
    }
}

impl std::fmt::Debug for ZMQRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ZMQRequest({})", self.handle)
    }
}

impl Request for ZMQRequest {
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.instance
            .borrow_mut()
            .send(NetworkerEvent::ExtendTimeout(
                self.handle,
                node_alias,
                timeout,
            ))
    }
    fn get_nodes(&self) -> Option<Nodes> {
        // FIXME send in Init
        unimplemented!()
    }
    fn get_stream<'a>(&'a mut self) -> BoxStream<'a, HandlerEvent> {
        self.events.by_ref().boxed()
    }
    fn get_timing(&self) -> Option<HashMap<String, f32>> {
        None
    }
    fn is_active(&self) -> bool {
        self.active
    }
    fn send_to_all(&self, timeout: RequestTimeout) -> LedgerResult<()> {
        self.instance.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            DispatchTarget::AllNodes,
            timeout,
        ))
    }
    fn send_to_any(&self, timeout: RequestTimeout) -> LedgerResult<()> {
        self.instance.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            DispatchTarget::AnyNode,
            timeout,
        ))
    }
    fn send_to(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.instance.borrow_mut().send(NetworkerEvent::Dispatch(
            self.handle,
            DispatchTarget::SelectNode(node_alias),
            timeout,
        ))
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
    ) -> LocalBoxFuture<'a, LedgerResult<RefCell<Box<dyn Request>>>>;
}

pub struct ZMQNetworker {
    id: NetworkerHandle,
    instance: Option<Rc<RefCell<ZMQNetworkerInstance>>>,
}

impl Networker for ZMQNetworker {
    fn new(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
    ) -> LedgerResult<Self> {
        let id = NetworkerHandle::next();
        let (nodes, remotes) = _get_nodes_and_remotes(transactions, config.protocol_version)?;
        let mut result = ZMQNetworker { id, instance: None };
        result.start(config, nodes, remotes, preferred_nodes);
        Ok(result)
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

    /*
    fn add_request(&mut self, request: PoolRequest) -> LedgerResult<()> {
        if let Some(worker) = &self.worker {
            self.req_send.send(request).map_err(|err| {
                LedgerError::from_msg(LedgerErrorKind::InvalidState, err.to_string())
            })?;
            worker.thread().unpark();
            Ok(())
        } else {
            Err(err_msg(
                LedgerErrorKind::InvalidState,
                "Networker not running",
            ))
        }
    }*/

    fn create_request<'a>(
        &'a mut self,
        message: &Message,
    ) -> LocalBoxFuture<'a, LedgerResult<RefCell<Box<dyn Request>>>> {
        let req_id = message.request_id().unwrap_or("".to_owned());
        let req_json = serde_json::to_string(&message)
            .map_err(|err| format!("Cannot serialize Request: {:?}", err));
        let instance = self.instance.clone();
        lazy(move |_| {
            if let Some(instance) = instance {
                trace!("hi from closure");
                let (tx, rx) = channel(0);
                let req_json =
                    req_json.map_err(|err| err_msg(LedgerErrorKind::InvalidState, err))?;
                let handle = RequestHandle::next();
                instance
                    .borrow_mut()
                    .send(NetworkerEvent::NewRequest(handle, req_id, req_json, tx))?;
                Ok(RefCell::new(
                    Box::new(ZMQRequest::new(handle, rx, instance.clone())) as Box<dyn Request>,
                ))
            } else {
                Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Networker not running",
                ))
            }
        })
        .boxed_local()
    }
}

impl ZMQNetworker {
    fn start(
        &mut self,
        config: PoolConfig,
        nodes: Nodes,
        remotes: Vec<RemoteNode>,
        preferred_nodes: Vec<String>,
    ) {
        let mut weights: Vec<(usize, f32)> = (0..remotes.len()).map(|idx| (idx, 1.0)).collect();
        for name in preferred_nodes {
            if let Some(index) = remotes.iter().position(|node| node.name == name) {
                weights[index] = (index, 2.0);
            }
        }
        let (cmd_send, cmd_recv) = _create_pair_of_sockets(&format!("zmqnet_{}", self.id.value()));
        let (req_send, req_recv) = mpsc::channel::<NetworkerEvent>();
        let worker = thread::spawn(move || {
            let mut zmq_thread =
                ZMQThread::new(config, cmd_recv, req_recv, nodes, remotes, weights);
            zmq_thread.work().map_err(map_err_err!());
            // FIXME send pool event when networker exits
            trace!("zmq worker exited")
        });
        self.instance
            .replace(Rc::new(RefCell::new(ZMQNetworkerInstance::new(
                cmd_send, req_send, worker,
            ))));
    }
}

struct ZMQNetworkerInstance {
    cmd_send: zmq::Socket,
    evt_send: mpsc::Sender<NetworkerEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl ZMQNetworkerInstance {
    fn new(
        cmd_send: zmq::Socket,
        evt_send: mpsc::Sender<NetworkerEvent>,
        worker: thread::JoinHandle<()>,
    ) -> Self {
        Self {
            cmd_send,
            evt_send,
            worker: Some(worker),
        }
    }

    fn send(&self, event: NetworkerEvent) -> LedgerResult<()> {
        self.evt_send
            .send(event)
            .to_result(LedgerErrorKind::InvalidState, "Error sending request")?;
        // stop waiting on current sockets
        self.cmd_send.send("", 0).to_result(
            LedgerErrorKind::InvalidState,
            "Error sending networker command",
        )
    }
}

impl Drop for ZMQNetworkerInstance {
    // FIXME call send with Exit event
    fn drop(&mut self) {
        if let Err(_) = self.cmd_send.send("exit", 0) {
            trace!("Networker command socket already closed")
        } else {
            trace!("Networker thread told to exit")
        }
        if let Some(worker) = self.worker.take() {
            info!("Drop networker thread");
            worker.join().unwrap()
        }
    }
}

struct ZMQThread {
    config: PoolConfig,
    cmd_recv: zmq::Socket,
    evt_recv: mpsc::Receiver<NetworkerEvent>,
    nodes: Nodes,
    remotes: Vec<RemoteNode>,
    weights: Vec<(usize, f32)>,
    requests: BTreeMap<RequestHandle, PendingRequest>,
    last_connection: Option<ZMQConnectionHandle>,
    pool_connections: BTreeMap<ZMQConnectionHandle, ZMQConnection>,
}

impl ZMQThread {
    pub fn new(
        config: PoolConfig,
        cmd_recv: zmq::Socket,
        evt_recv: mpsc::Receiver<NetworkerEvent>,
        nodes: Nodes,
        remotes: Vec<RemoteNode>,
        weights: Vec<(usize, f32)>,
    ) -> Self {
        ZMQThread {
            config,
            cmd_recv,
            evt_recv,
            nodes,
            remotes,
            weights,
            requests: BTreeMap::new(),
            last_connection: None,
            pool_connections: BTreeMap::new(),
        }
    }

    pub fn work(&mut self) -> Result<(), String> {
        loop {
            while self.try_receive_request()? {}
            match self.poll_connections()? {
                PollResult::NoSockets => {
                    // wait until a request is received
                    // thread::park()
                }
                PollResult::Events(events) => {
                    for (conn_id, event) in events {
                        self.process_event(conn_id, event)
                    }
                }
                PollResult::Exit => {
                    trace!("Networker thread ended");
                    break;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn poll_connections(&mut self) -> Result<PollResult, String> {
        let (conn_idx, mut poll_items) = self.get_poll_items();
        let (last_req, timeout) = self.get_timeout();
        let mut events = vec![];
        poll_items.push(self.cmd_recv.as_poll_item(zmq::POLLIN));
        let poll_res = zmq::poll(&mut poll_items, ::std::cmp::max(timeout, 0))
            .map_err(|err| format!("Error polling ZMQ sockets: {:?}", err))?;
        if poll_res == 0 {
            if let Some((conn_id, req_id, node_alias)) = last_req {
                events.push((conn_id, NodeEvent::Timeout(req_id, node_alias)));
            }
        } else {
            self.fetch_events_into(conn_idx.as_slice(), poll_items.as_slice(), &mut events);
        }
        if poll_items[poll_items.len() - 1].is_readable() {
            if let Ok(Ok(msg)) = self.cmd_recv.recv_string(zmq::DONTWAIT) {
                if msg == "exit" {
                    return Ok(PollResult::Exit);
                }
            } else {
                // command socket failed
                return Ok(PollResult::Exit);
            }
        }
        if events.len() > 0 {
            trace!("Got {} events", events.len());
            return Ok(PollResult::Events(events));
        } else {
            //print!(".")
        }
        if poll_items.len() == 1 {
            return Ok(PollResult::NoSockets);
        }
        return Ok(PollResult::Default);
    }

    fn process_event(&mut self, conn_id: ZMQConnectionHandle, event: NodeEvent) {
        let (req_id, fwd) = match event {
            NodeEvent::Reply(req_id, message, node_alias, time) => (
                req_id,
                HandlerEvent::Received(node_alias.clone(), message, time),
            ),
            NodeEvent::Timeout(req_id, node_alias) => {
                (req_id, HandlerEvent::Timeout(node_alias.clone()))
            }
        };
        if let Some(handle) = self.select_request(conn_id, req_id.clone()) {
            self.process_reply(handle, fwd)
        } else {
            trace!("Unknown request ID: {}", req_id)
        }
        /* match &message {
                Message::LedgerStatus(_) | Message::ConsistencyProof(_) => {
                    let handle = self.select_request(conn_id, "".to_string());
                    self.process_reply(
                        handle,
                        HandlerEvent::Received(node_alias.clone(), message, time),
                    )
                }
                Message::CatchupReq(_) => {
                    warn!("Ignoring catchup request");
                }
                Message::CatchupRep(_) => {
                    let handle = self.select_request(conn_id, "".to_string());
                    self.process_reply(
                        handle,
                        HandlerEvent::Received(node_alias.clone(), message, time),
                    )
                }
                _ => {
                    trace!("Unhandled message {:?}", message);
                }
            },
            NodeEvent::Timeout(req_id, node_alias) => {

                self.process_reply(req_id, HandlerEvent::Timeout(node_alias.clone()))
            }
        }*/
    }

    fn select_request(
        &self,
        conn_id: ZMQConnectionHandle,
        sub_id: String,
    ) -> Option<RequestHandle> {
        self.requests.iter().find_map(|(handle, req)| {
            if req.conn_id == conn_id && req.sub_id == sub_id {
                Some(*handle)
            } else {
                None
            }
        })
    }

    /*fn send_update(&self, event: PoolEvent) {
        if let Err(_) = self.sender.send(event) {
            trace!("Lost pool update sender")
        }
    }*/

    fn clean_timeout(
        &mut self,
        conn_id: ZMQConnectionHandle,
        sub_id: &String,
        node_alias: Option<String>,
    ) {
        if let Some(delete) = self.pool_connections.get(&conn_id).and_then(|pc| {
            pc.clean_timeout(&sub_id, node_alias);
            if pc.is_orphaned() {
                Some(conn_id)
            } else {
                None
            }
        }) {
            trace!("removing pool connection {}", conn_id);
            self.pool_connections.remove(&delete);
        }
    }

    fn process_reply(&mut self, handle: RequestHandle, event: HandlerEvent) {
        if let Some(req) = self.requests.get_mut(&handle) {
            // FIXME - detect cancelled request and clean up
            req.sender.try_send(event).unwrap()
        /*trace!("got update {:?}", update);
        if let Ok(update) = update {
            match update {
                HandlerUpdate::Continue => (),
                HandlerUpdate::ExtendTimeout(alias, timeout) => trace!("Extend timeout"),
                HandlerUpdate::Finish(opt_event) => {
                    if opt_event.is_some() {
                        self.send_update(opt_event.unwrap())
                    }
                    self.remove_request(req_id)
                }
                HandlerUpdate::Resend(_timeout) => (),
            }
        } else {
            // self.send_update(PoolEvent::RequestError(req_id, update));
            self.remove_request(req_id)
        }*/
        } else {
            trace!("Request ID not found: {}", handle);
        }
    }

    fn try_receive_request(&mut self) -> Result<bool, String> {
        let event = match self.evt_recv.try_recv() {
            Ok(request) => request,
            Err(mpsc::TryRecvError::Empty) => return Ok(false),
            Err(err) => return Err(err.to_string()),
        };
        match event {
            NetworkerEvent::NewRequest(handle, sub_id, body, sender) => {
                // FIXME improve error handling
                trace!("New request {}", handle);
                let nodes = self.nodes.clone();
                let pending = self.add_request(handle, sub_id, body, sender).unwrap();
                pending
                    .sender
                    .try_send(HandlerEvent::Init(nodes))
                    .map_err(|err| err.to_string())?;
                Ok(true)
            }
            NetworkerEvent::CancelRequest(handle) => {
                self.remove_request(handle);
                Ok(true)
            }
            NetworkerEvent::Dispatch(handle, target, timeout) => {
                // FIXME improve error handling
                trace!("Dispatch {} {:?}", handle, target);
                self.dispatch_request(handle, target, timeout).unwrap();
                Ok(true)
            }
            NetworkerEvent::ExtendTimeout(handle, node_alias, timeout) => {
                self.extend_timeout(handle, node_alias, timeout).unwrap();
                Ok(true)
            }
        }
    }

    fn add_request(
        &mut self,
        handle: RequestHandle,
        sub_id: String,
        body: String,
        sender: Sender<HandlerEvent>,
    ) -> LedgerResult<&mut PendingRequest> {
        let conn_id = self.get_active_connection(self.last_connection, Some(sub_id.clone()))?;
        // FIXME add sub_id to `seen` for connection - tracked here or at connection level?
        let pending = PendingRequest {
            conn_id,
            send_index: 0,
            sender,
            sub_id,
            body,
        };
        self.requests.insert(handle, pending);
        self.requests.get_mut(&handle).ok_or(err_msg(
            LedgerErrorKind::InvalidState,
            "Error adding request",
        ))
        /*let (init_target, subscribe, init_timeout) = handler.borrow().get_target();
        let pending = PendingRequest {
            msg: req_json.clone(),
            send_index: 0,
            subscribe,
            handler,
        };
        self.requests.insert(req_id.clone(), pending);
        let node_indexes = self.select_targets(init_target)?;
        let timeout = self.select_timeout(init_timeout);
        self.send_request(
            req_id,
            req_json.clone(),
            timeout,
            self.last_connection,
            node_indexes,
            0,
        )*/
    }

    fn remove_request(&mut self, handle: RequestHandle) {
        if let Some(req) = self.requests.remove(&handle) {
            self.clean_timeout(req.conn_id, &req.sub_id, None)
        }
    }

    fn dispatch_request(
        &mut self,
        handle: RequestHandle,
        target: DispatchTarget,
        timeout: RequestTimeout,
    ) -> LedgerResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                let timeout = timeout.expand(&self.config);
                match target {
                    DispatchTarget::AllNodes => {
                        for i in 0..self.nodes.len() {
                            conn.send_request(
                                request.sub_id.clone(),
                                request.body.clone(),
                                timeout,
                                request.send_index,
                            )?;
                            request.send_index += 1
                        }
                    }
                    DispatchTarget::AnyNode => {
                        conn.send_request(
                            request.sub_id.clone(),
                            request.body.clone(),
                            timeout,
                            request.send_index,
                        )?;
                        request.send_index += 1
                    }
                    DispatchTarget::SelectNode(named) => {
                        if let Some(index) = conn.get_send_index(named.as_str()) {
                            conn.send_request(
                                request.sub_id.clone(),
                                request.body.clone(),
                                timeout,
                                index,
                            )?;
                            request.send_index = index + 1
                        } else {
                            warn!("Cannot send to unknown node alias: {}", named)
                        }
                    }
                }
            } else {
                warn!("Pool connection expired for request: {}", handle)
            }
        } else {
            warn!("Unknown request ID for dispatch: {}", handle)
        }
        Ok(())
    }

    fn extend_timeout(
        &mut self,
        handle: RequestHandle,
        node_alias: String,
        timeout: RequestTimeout,
    ) -> LedgerResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get(&request.conn_id) {
                conn.extend_timeout(
                    request.sub_id.as_str(),
                    node_alias.as_str(),
                    timeout.expand(&self.config),
                )
            } else {
                warn!("Pool connection expired for extend timeout: {}", handle)
            }
        } else {
            warn!("Unknown request ID for extend timeout: {}", handle)
        }
        Ok(())
    }

    fn get_active_connection(
        &mut self,
        conn_id: Option<ZMQConnectionHandle>,
        sub_id: Option<String>,
    ) -> LedgerResult<ZMQConnectionHandle> {
        let conn = conn_id
            .and_then(|conn_id| self.pool_connections.get(&conn_id))
            .filter(|conn| {
                conn.is_active() && conn.req_cnt < self.config.conn_request_limit
                /* && !conn.seen(sub_id) */
            });
        if conn.is_none() {
            let conn = ZMQConnection::new(self.remotes.clone(), self.config.conn_active_timeout);
            trace!("Created new pool connection");
            let pc_id = ZMQConnectionHandle::next();
            self.pool_connections.insert(pc_id, conn);
            self.last_connection.replace(pc_id);
            Ok(pc_id)
        } else {
            Ok(conn_id.unwrap())
        }
    }

    /*fn select_targets(&self, target: PoolRequestTarget) -> LedgerResult<Vec<usize>> {
        let mut targets = vec![];
        let count_all = self.remotes.len();
        match target {
            PoolRequestTarget::AllNodes => {
                // order doesn't matter
                targets.extend(0..count_all);
                Ok(targets)
            }
            PoolRequestTarget::AnyNodes(count) => {
                let count = ::std::cmp::min(count, count_all);
                let rng = &mut thread_rng();
                let mut weights = self.weights.to_vec();
                for _ in 0..count {
                    let (index, _) = *weights.choose_weighted(rng, |item| item.1).unwrap();
                    targets.push(index);
                    weights[index] = (index, 0.0)
                }
                Ok(targets)
            }
            PoolRequestTarget::SelectNodes(select_nodes) => {
                /*let nodes = &self.nodes;
                let found_nodes = select_nodes
                    .iter()
                    .cloned()
                    .filter(|node| nodes.contains_key(node))
                    .collect::<Vec<String>>();
                if !found_nodes.is_empty() {
                    Ok(conn.send_request(NetworkerEvent::SendAllRequest(
                        request.req_json.clone(),
                        req_id.clone(),
                        request.init_timeout,
                        Some(select_nodes.clone()),
                    ))?)
                } else {
                    Err(err_msg(
                        LedgerErrorKind::InvalidStructure,
                        format!(
                            "There is no known node in list to send {:?}, known nodes are {:?}",
                            select_nodes,
                            self.nodes.keys()
                        ),
                    ))
                }*/
                Ok(vec![0 as usize])
            }
        }
    }*/

    /*fn send_request(
        &mut self,
        req_id: String,
        msg: String,
        timeout: i64,
        last_conn: Option<ZMQConnectionHandle>,
        node_indexes: Vec<usize>,
        send_index: usize,
    ) -> LedgerResult<()> {
        trace!("Send request to {:?}", node_indexes);
        let conn_id = self.get_active_connection(last_conn);
        let mut send_index = send_index;
        for idx in node_indexes {
            let when = SystemTime::now();
            let name = {
                let conn = unwrap_opt_or_return!(
                    self.pool_connections.get_mut(&conn_id),
                    Err(err_msg(
                        LedgerErrorKind::InvalidState,
                        "Pool connection not found for dispatch"
                    ))
                );
                conn.send_request(req_id.clone(), msg.clone(), timeout, idx)?
            };
            self.process_reply(req_id.clone(), HandlerEvent::Sent(name, when, send_index));
            send_index += 1
        }
        Ok(())
    }*/

    fn fetch_events_into(
        &self,
        conn_idx: &[(ZMQConnectionHandle, usize)],
        poll_items: &[PollItem],
        events: &mut Vec<(ZMQConnectionHandle, NodeEvent)>,
    ) {
        let mut cnt = 0;
        // FIXME - can avoid duplicate lookups for the same connection
        conn_idx
            .iter()
            .zip(poll_items)
            .for_each(|((conn_id, index), poll_item)| {
                if let Some(conn) = self.pool_connections.get(conn_id) {
                    if let Some(event) = conn.fetch_event(*index, poll_item) {
                        events.push((*conn_id, event))
                    }
                } else {
                    trace!(
                        "Connection removed before socket could be read: {}",
                        conn_id
                    )
                }
            })
        /*events.extend(
            self.pool_connections
                .values()
                .map(|pc| {
                    let ocnt = cnt;
                    cnt += pc.sockets.iter().filter(|s| s.is_some()).count();
                    pc.fetch_events(&poll_items[ocnt..cnt])
                })
                .flat_map(|v| v.into_iter()),
        );*/
    }

    /*
    fn process_event(&mut self, pe: Option<NetworkerEvent>) -> Option<RequestEvent> {
        match pe.clone() {
            Some(NetworkerEvent::SendAllRequest(_, req_id, _, _))
            | Some(NetworkerEvent::SendOneRequest(_, req_id, _))
            | Some(NetworkerEvent::Resend(req_id, _)) => {
                let num = self.req_id_mappings.get(&req_id).copied().or_else(|| {
                    trace!("sending new request");
                    self.pool_connections
                        .iter()
                        .next_back()
                        .and_then(|(pc_idx, pc)| {
                            if pc.is_active()
                                && pc.req_cnt < self.conn_limit
                                && pc
                                    .nodes
                                    .iter()
                                    .collect::<HashSet<&RemoteNode>>()
                                    .eq(&self.nodes.iter().collect::<HashSet<&RemoteNode>>())
                            {
                                trace!("existing connection available");
                                Some(*pc_idx)
                            } else {
                                trace!("existing connection unavailable");
                                None
                            }
                        })
                });
                match num {
                    Some(idx) => {
                        trace!("send request in existing conn");

                        match self.pool_connections.get_mut(&idx) {
                            Some(pc) => pc.send_request(pe).expect("FIXME"),
                            None => error!("Pool Connection not found"),
                        }
                        self.req_id_mappings.insert(req_id.clone(), idx);
                    }
                    None => {
                        trace!("send request in new conn");
                        let pc_id = next_pool_connection_handle();
                        let mut pc = ZMQConnection::new(
                            self.nodes.clone(),
                            self.active_timeout,
                            self.preordered_nodes.clone(),
                        );
                        pc.send_request(pe).expect("FIXME");
                        self.pool_connections.insert(pc_id, pc);
                        self.req_id_mappings.insert(req_id.clone(), pc_id);
                    }
                }
                None
            }
            Some(NetworkerEvent::NodesStateUpdated(nodes)) => {
                trace!("ZMQNetworker::process_event: nodes_updated {:?}", nodes);
                self.nodes = nodes;
                None
            }
            Some(NetworkerEvent::ExtendTimeout(req_id, node_alias, timeout)) => {
                self.req_id_mappings.get(&req_id).map(|idx| {
                    self.pool_connections.get(idx).map(|pc| {
                        pc.extend_timeout(&req_id, &node_alias, timeout);
                    });
                });
                None
            }
            Some(NetworkerEvent::CleanTimeout(req_id, node_alias)) => {
                {
                    let idx_pc_to_delete = self.req_id_mappings.get(&req_id).and_then(|idx| {
                        let delete = self
                            .pool_connections
                            .get(idx)
                            .map(|pc| {
                                pc.clean_timeout(&req_id, node_alias.clone());
                                pc.is_orphaned()
                            })
                            .unwrap_or(false);

                        if delete {
                            Some(idx)
                        } else {
                            None
                        }
                    });
                    if let Some(idx) = idx_pc_to_delete {
                        trace!("removing pool connection {}", idx);
                        self.pool_connections.remove(idx);
                    }
                }

                if node_alias.is_none() {
                    self.req_id_mappings.remove(&req_id);
                }

                None
            }
            Some(NetworkerEvent::Timeout) => {
                let pc_to_delete: Vec<u32> = self
                    .pool_connections
                    .iter()
                    .filter(|(_, v)| v.is_orphaned())
                    .map(|(k, _)| *k)
                    .collect();
                pc_to_delete.iter().for_each(|idx| {
                    trace!("removing pool connection {}", idx);
                    self.pool_connections.remove(idx);
                });
                None
            }
            _ => None,
        }
    }
    */

    fn get_timeout(&self) -> (Option<(ZMQConnectionHandle, String, String)>, i64) {
        self.pool_connections
            .iter()
            .map(|(handle, conn)| conn.get_timeout(*handle))
            .min_by(|&(_, val1), &(_, val2)| val1.cmp(&val2))
            .unwrap_or((None, 0))
    }

    fn get_poll_items(&self) -> (Vec<(ZMQConnectionHandle, usize)>, Vec<PollItem>) {
        self.pool_connections
            .iter()
            .flat_map(|(handle, conn)| conn.get_poll_items(*handle))
            .unzip()
    }
}

pub struct ZMQConnection {
    nodes: Vec<RemoteNode>,
    sockets: Vec<Option<ZSocket>>,
    ctx: zmq::Context,
    key_pair: zmq::CurveKeyPair,
    timeouts: RefCell<HashMap<(String, String), Instant>>,
    time_created: Instant,
    req_cnt: usize,
    active_timeout: i64,
}

impl ZMQConnection {
    fn new(nodes: Vec<RemoteNode>, active_timeout: i64) -> Self {
        trace!("ZMQConnection::new: from nodes {:?}", nodes);

        let mut sockets: Vec<Option<ZSocket>> = Vec::with_capacity(nodes.len());

        for _ in 0..nodes.len() {
            sockets.push(None);
        }

        Self {
            nodes,
            sockets,
            ctx: zmq::Context::new(),
            key_pair: zmq::CurveKeyPair::new().expect("FIXME"),
            time_created: Instant::now(),
            timeouts: RefCell::new(HashMap::new()),
            req_cnt: 0,
            active_timeout,
        }
    }

    fn fetch_event(&self, index: usize, poll_item: &zmq::PollItem) -> Option<NodeEvent> {
        if let (&Some(ref s), rn) = (&self.sockets[index], &self.nodes[index]) {
            if poll_item.is_readable() {
                if let Ok(Ok(msg)) = s.recv_string(zmq::DONTWAIT) {
                    match Message::from_raw_str(msg.as_str()) {
                        Ok(message) => {
                            return Some(NodeEvent::Reply(
                                message.request_id().unwrap_or("".to_string()),
                                message,
                                rn.name.clone(),
                                SystemTime::now(),
                            ))
                        }
                        Err(err) => error!("Error parsing received message: {:?}", err),
                    }
                }
            }
        }
        None
    }

    fn get_poll_items(
        &self,
        handle: ZMQConnectionHandle,
    ) -> Vec<((ZMQConnectionHandle, usize), PollItem)> {
        self.sockets
            .iter()
            .enumerate()
            .flat_map(|(idx, zs): (usize, &Option<ZSocket>)| {
                zs.as_ref()
                    .map(|zs| ((handle, idx), zs.as_poll_item(zmq::POLLIN)))
            })
            .collect()
    }

    fn get_timeout(
        &self,
        handle: ZMQConnectionHandle,
    ) -> (Option<(ZMQConnectionHandle, String, String)>, i64) {
        let now = Instant::now();
        let (target, timeout) = {
            if let Some((&(ref req_id, ref node_alias), timeout)) = self
                .timeouts
                .borrow()
                .iter()
                .min_by(|&(_, ref val1), &(_, ref val2)| val1.cmp(&val2))
            {
                (
                    Some((handle, req_id.to_string(), node_alias.to_string())),
                    *timeout,
                )
            } else {
                (
                    None,
                    self.time_created
                        + Duration::new(::std::cmp::max(self.active_timeout, 0) as u64, 0),
                )
            }
        };
        (
            target,
            timeout
                .checked_duration_since(now)
                .unwrap_or(Duration::new(0, 0))
                .as_millis() as i64,
        )
    }

    fn is_active(&self) -> bool {
        trace!(
            "is_active >> time worked: {:?}",
            Instant::now() - self.time_created
        );
        let res = self.time_created.elapsed()
            > Duration::from_secs(::std::cmp::max(self.active_timeout, 0) as u64);
        trace!("is_active << {}", res);
        res
    }

    fn get_send_index(&self, node_alias: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.name == node_alias)
    }

    fn send_request(
        &mut self,
        req_id: String,
        msg: String,
        timeout: i64,
        send_index: usize,
    ) -> LedgerResult<String> {
        trace!("send_request >> req_id: {} idx: {}", req_id, send_index);
        let node_index = send_index % self.nodes.len();
        let name = self.nodes[node_index].name.clone();
        {
            let s = self._get_socket(node_index)?;
            s.send(&msg, zmq::DONTWAIT)?;
        }
        self.timeouts.borrow_mut().insert(
            (req_id.clone(), name.clone()),
            Instant::now() + Duration::from_secs(::std::cmp::max(timeout, 0) as u64),
        );
        self.req_cnt += 1;
        trace!("send_request <<");
        Ok(name)
    }

    fn extend_timeout(&self, req_id: &str, node_alias: &str, extended_timeout: i64) {
        if let Some(timeout) = self
            .timeouts
            .borrow_mut()
            .get_mut(&(req_id.to_string(), node_alias.to_string()))
        {
            *timeout =
                Instant::now() + Duration::from_secs(::std::cmp::max(extended_timeout, 0) as u64);
        } else {
            debug!("late REQACK for req_id {}, node {}", req_id, node_alias);
        }
    }

    fn clean_timeout(&self, req_id: &str, node_alias: Option<String>) {
        match node_alias {
            Some(node_alias) => {
                self.timeouts
                    .borrow_mut()
                    .remove(&(req_id.to_string(), node_alias));
            }
            None => {
                let keys_to_remove: Vec<(String, String)> = self
                    .timeouts
                    .borrow()
                    .keys()
                    .cloned()
                    .filter(|&(ref req_id_timeout, _)| req_id == req_id_timeout)
                    .collect();
                keys_to_remove.iter().for_each(|key| {
                    self.timeouts.borrow_mut().remove(key);
                });
            }
        }
    }

    fn has_active_requests(&self) -> bool {
        !self.timeouts.borrow().is_empty()
    }

    fn is_orphaned(&self) -> bool {
        !self.is_active() && !self.has_active_requests()
    }

    fn _get_socket(&mut self, idx: usize) -> LedgerResult<&ZSocket> {
        if self.sockets[idx].is_none() {
            debug!("_get_socket: open new socket for node {}", idx);
            let s: ZSocket = self.nodes[idx].connect(&self.ctx, &self.key_pair)?;
            self.sockets[idx] = Some(s)
        }
        Ok(self.sockets[idx].as_ref().unwrap())
    }
}

impl RemoteNode {
    fn connect(&self, ctx: &zmq::Context, key_pair: &zmq::CurveKeyPair) -> LedgerResult<ZSocket> {
        let s = ctx.socket(zmq::SocketType::DEALER)?;
        s.set_identity(base64::encode(&key_pair.public_key).as_bytes())?;
        s.set_curve_secretkey(&key_pair.secret_key)?;
        s.set_curve_publickey(&key_pair.public_key)?;
        s.set_curve_serverkey(
            zmq::z85_encode(self.public_key.as_slice())
                .to_result(
                    LedgerErrorKind::InvalidStructure,
                    "Can't encode server key as z85",
                )? // FIXME: review kind
                .as_bytes(),
        )?;
        s.set_linger(0)?; //TODO set correct timeout
        s.connect(&self.zaddr)?;
        Ok(s)
    }
}

fn _create_pair_of_sockets(addr: &str) -> (zmq::Socket, zmq::Socket) {
    let zmq_ctx = zmq::Context::new();
    let send_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();
    let recv_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();

    let inproc_sock_name: String = format!("inproc://{}", addr);
    recv_cmd_sock.bind(inproc_sock_name.as_str()).unwrap();
    send_cmd_sock.connect(inproc_sock_name.as_str()).unwrap();
    (send_cmd_sock, recv_cmd_sock)
}

fn _get_nodes_and_remotes(
    transactions: Vec<String>,
    protocol_version: ProtocolVersion,
) -> LedgerResult<(Nodes, Vec<RemoteNode>)> {
    let txn_map = build_node_state_from_json(transactions, protocol_version)?;
    Ok(txn_map
        .iter()
        .map(|(dest, txn)| {
            let node_alias = txn.txn.data.data.alias.clone();

            let node_verkey = dest.as_str().from_base58().to_result(
                LedgerErrorKind::InvalidStructure,
                "Invalid field dest in genesis transaction",
            )?;

            let node_verkey = crypto::import_verkey(node_verkey)
                .and_then(|vk| crypto::vk_to_curve25519(vk))
                .to_result(
                    LedgerErrorKind::InvalidStructure,
                    "Invalid field dest in genesis transaction",
                )?;

            if txn.txn.data.data.services.is_none()
                || !txn
                    .txn
                    .data
                    .data
                    .services
                    .as_ref()
                    .unwrap()
                    .contains(&"VALIDATOR".to_string())
            {
                return Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Node is not a validator",
                )); // FIXME: review error kind
            }

            let address = match (&txn.txn.data.data.client_ip, &txn.txn.data.data.client_port) {
                (&Some(ref client_ip), &Some(ref client_port)) => {
                    format!("tcp://{}:{}", client_ip, client_port)
                }
                _ => {
                    return Err(err_msg(
                        LedgerErrorKind::InvalidState,
                        "Client address not found",
                    ))
                }
            };

            let remote = RemoteNode {
                name: node_alias.clone(),
                public_key: node_verkey[..].to_vec(),
                // TODO:FIXME
                zaddr: address,
                is_blacklisted: false,
            };

            let verkey: Option<BlsVerKey> = match txn.txn.data.data.blskey {
                Some(ref blskey) => {
                    let key = blskey.as_str().from_base58().to_result(
                        LedgerErrorKind::InvalidStructure,
                        "Invalid field blskey in genesis transaction",
                    )?;

                    Some(BlsVerKey::from_bytes(&key).to_result(
                        LedgerErrorKind::InvalidStructure,
                        "Invalid field blskey in genesis transaction",
                    )?)
                }
                None => None,
            };
            Ok(((node_alias, verkey), remote))
        })
        .fold((HashMap::new(), vec![]), |(mut map, mut vec), res| {
            match res {
                Err(e) => {
                    error!("Error during retrieving nodes: {:?}", e);
                }
                Ok(((alias, verkey), remote)) => {
                    map.insert(alias.clone(), verkey);
                    vec.push(remote);
                }
            }
            (map, vec)
        }))
}

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
    ) -> LocalBoxFuture<'a, LedgerResult<RefCell<Box<dyn Request>>>> {
        unimplemented!()
    }
}

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
