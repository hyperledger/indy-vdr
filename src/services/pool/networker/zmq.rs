extern crate zmq;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use futures::channel::mpsc::{channel, Sender};
use futures::future::{lazy, FutureExt, LocalBoxFuture};

use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::FromBase58;
use crate::utils::crypto;
use crate::utils::error::prelude::*;

use super::super::merkle_tree_factory::build_node_state_from_json;
use super::super::types::{Message, Nodes, PoolConfig};
use super::{
    Networker, NetworkerEvent, NetworkerHandle, NetworkerRequest, NetworkerRequestImpl,
    NetworkerSender, RequestDispatchTarget, RequestExtEvent, RequestHandle, RequestTimeout,
};

use base64;
use ursa::bls::VerKey as BlsVerKey;

use zmq::PollItem;
use zmq::Socket as ZSocket;

new_handle_type!(ZMQConnectionHandle, PHC_COUNTER);

pub struct ZMQNetworker {
    id: NetworkerHandle,
    instance: Option<Rc<RefCell<ZMQNetworkerInstance>>>,
    protocol_version: ProtocolVersion,
    nodes_count: usize,
}

impl Networker for ZMQNetworker {
    fn new(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
    ) -> LedgerResult<Self> {
        let id = NetworkerHandle::next();
        let (nodes, remotes) = _get_nodes_and_remotes(transactions, config.protocol_version)?;
        let mut result = ZMQNetworker {
            id,
            instance: None,
            protocol_version: config.protocol_version,
            nodes_count: nodes.len(),
        };
        result.start(config, nodes, remotes, preferred_nodes);
        Ok(result)
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

    fn create_request<'a>(
        &'a self,
        message: &Message,
    ) -> LocalBoxFuture<'a, LedgerResult<Box<dyn NetworkerRequest>>> {
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
                Ok(
                    Box::new(NetworkerRequestImpl::new(handle, rx, instance.clone()))
                        as Box<dyn NetworkerRequest>,
                )
            } else {
                Err(err_msg(
                    LedgerErrorKind::InvalidState,
                    "Networker not running",
                ))
            }
        })
        .boxed_local()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        return self.protocol_version;
    }

    fn nodes_count(&self) -> usize {
        return self.nodes_count;
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
            if let Err(err) = zmq_thread.work() {
                error!("ZMQ worker exited with error: {}", err.to_string())
            } else {
                trace!("ZMQ worker exited");
            }
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
}

impl NetworkerSender for ZMQNetworkerInstance {
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
                RequestExtEvent::Received(node_alias.clone(), message, time),
            ),
            NodeEvent::Timeout(req_id, node_alias) => {
                (req_id, RequestExtEvent::Timeout(node_alias.clone()))
            }
        };
        if let Some(handle) = self.select_request(conn_id, req_id.clone()) {
            self.process_reply(handle, fwd)
        } else {
            trace!("Unknown request ID: {}", req_id)
        }
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

    fn process_reply(&mut self, handle: RequestHandle, event: RequestExtEvent) {
        if let Some(req) = self.requests.get_mut(&handle) {
            // FIXME - detect cancelled request and clean up
            if let Err(err) = req.sender.try_send(event) {
                if err.is_full() {
                    trace!("Removing request: buffer full {}", handle);
                } else if err.is_disconnected() {
                    trace!("Removing request: sender disconnected {}", handle);
                } else {
                    trace!("Removing request: send error {} {:?}", handle, err);
                }
                self.remove_request(handle)
            }
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
                    .try_send(RequestExtEvent::Init(nodes))
                    .map_err(|err| err.to_string())?;
                trace!("sent init");
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
        sender: Sender<RequestExtEvent>,
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
    }

    fn remove_request(&mut self, handle: RequestHandle) {
        if let Some(req) = self.requests.remove(&handle) {
            self.clean_timeout(req.conn_id, &req.sub_id, None)
        }
    }

    fn dispatch_request(
        &mut self,
        handle: RequestHandle,
        target: RequestDispatchTarget,
        timeout: RequestTimeout,
    ) -> LedgerResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                let timeout = timeout.expand(&self.config);
                match target {
                    RequestDispatchTarget::AllNodes => {
                        for _ in 0..self.nodes.len() {
                            conn.send_request(
                                request.sub_id.clone(),
                                request.body.clone(),
                                timeout,
                                request.send_index,
                            )?;
                            request.send_index += 1
                        }
                    }
                    RequestDispatchTarget::AnyNode => {
                        conn.send_request(
                            request.sub_id.clone(),
                            request.body.clone(),
                            timeout,
                            request.send_index,
                        )?;
                        request.send_index += 1
                    }
                    RequestDispatchTarget::SelectNode(named) => {
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
                conn.is_active()
                    && conn.req_cnt < self.config.conn_request_limit
                    && sub_id
                        .map(|sub_id| !conn.seen(sub_id.as_str()))
                        .unwrap_or(true)
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

    fn fetch_events_into(
        &self,
        conn_idx: &[(ZMQConnectionHandle, usize)],
        poll_items: &[PollItem],
        events: &mut Vec<(ZMQConnectionHandle, NodeEvent)>,
    ) {
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
    }

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
    req_log: HashSet<String>,
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
            req_log: HashSet::new(),
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
        self.req_log.insert(req_id);
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

    fn seen(&self, req_id: &str) -> bool {
        self.req_log.contains(req_id)
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

#[derive(Debug)]
enum PollResult {
    Default,
    Events(Vec<(ZMQConnectionHandle, NodeEvent)>),
    NoSockets,
    Exit,
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

#[derive(Debug)]
struct PendingRequest {
    conn_id: ZMQConnectionHandle,
    send_index: usize,
    sender: Sender<RequestExtEvent>,
    sub_id: String,
    body: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RemoteNode {
    pub name: String,
    pub public_key: Vec<u8>,
    pub zaddr: String,
    pub is_blacklisted: bool,
}
