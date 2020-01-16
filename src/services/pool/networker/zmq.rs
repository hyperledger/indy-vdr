extern crate zmq;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use base64;
use futures::channel::mpsc::Sender;
use rand::seq::SliceRandom;
use ursa::bls::VerKey as BlsVerKey;

use zmq::PollItem;
use zmq::Socket as ZSocket;

use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::{FromBase58, ToBase58};
use crate::utils::crypto;
use crate::utils::error::prelude::*;

use super::genesis::build_node_state_from_json;
use super::pool::Pool;
use super::types::{Message, Nodes, PoolConfig};
use super::{
    Networker, NetworkerEvent, RequestDispatchTarget, RequestExtEvent, RequestHandle,
    RequestTimeout,
};

new_handle_type!(ZMQSocketHandle, ZSC_COUNTER);

new_handle_type!(ZMQConnectionHandle, ZCH_COUNTER);

pub struct ZMQNetworker {
    cmd_send: zmq::Socket,
    evt_send: mpsc::Sender<NetworkerEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl ZMQNetworker {
    pub fn create_pool(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
    ) -> LedgerResult<Pool> {
        let (nodes, remotes) = _get_nodes_and_remotes(transactions, config.protocol_version)?;
        let mut weights: Vec<f32> = vec![1.0; remotes.len()];
        for name in preferred_nodes {
            if let Some(index) = remotes.iter().position(|node| node.name == name) {
                weights[index] = 2.0;
            }
        }
        let socket_handle = ZMQSocketHandle::next().value();
        let (cmd_send, cmd_recv) = _create_pair_of_sockets(&format!("zmqnet_{}", socket_handle));
        let (evt_send, evt_recv) = mpsc::channel::<NetworkerEvent>();
        let worker_nodes = nodes.clone();
        let worker = thread::spawn(move || {
            let mut zmq_thread =
                ZMQThread::new(config, cmd_recv, evt_recv, worker_nodes, remotes, weights);
            if let Err(err) = zmq_thread.work() {
                error!("ZMQ worker exited with error: {}", err.to_string())
            } else {
                trace!("ZMQ worker exited");
            }
        });
        let instance = Arc::new(RwLock::new(Self {
            cmd_send,
            evt_send,
            worker: Some(worker),
        }));
        Ok(Pool::new(config, instance, nodes))
    }
}

impl Networker for ZMQNetworker {
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

impl Drop for ZMQNetworker {
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
    weights: Vec<f32>,
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
        weights: Vec<f32>,
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
            self.clean_requests()?
        }
        Ok(())
    }

    fn poll_connections(&mut self) -> Result<PollResult, String> {
        let (conn_idx, mut poll_items) = self.get_poll_items();
        let (conn_req, timeout) = self.get_timeout();
        let mut events = vec![];
        poll_items.push(self.cmd_recv.as_poll_item(zmq::POLLIN));
        let poll_res = zmq::poll(&mut poll_items, ::std::cmp::max(timeout, 0))
            .map_err(|err| format!("Error polling ZMQ sockets: {:?}", err))?;
        if poll_res == 0 {
            if let Some((conn_id, last_req)) = conn_req {
                if let Some((req_id, node_alias)) = last_req {
                    events.push((conn_id, NodeEvent::RequestTimeout(req_id, node_alias)));
                } else {
                    events.push((conn_id, NodeEvent::Timeout()));
                }
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
        let (req_id, seen, fwd) = match event {
            NodeEvent::Reply(message, meta, node_alias, time) => {
                let req_id = meta.request_id().unwrap_or("".to_owned());
                (
                    req_id,
                    Some(node_alias.clone()),
                    RequestExtEvent::Received(node_alias, message, meta, time),
                )
            }
            NodeEvent::RequestTimeout(req_id, node_alias) => (
                req_id,
                Some(node_alias.clone()),
                RequestExtEvent::Timeout(node_alias),
            ),
            NodeEvent::Timeout() => {
                self.check_remove_connection(conn_id, None);
                return;
            }
        };
        if seen.is_some() {
            if let Some(conn) = self.pool_connections.get_mut(&conn_id) {
                conn.silence_timeout(&req_id, &seen.unwrap())
            }
        }
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

    fn process_reply(&mut self, handle: RequestHandle, event: RequestExtEvent) {
        if let Some(req) = self.requests.get_mut(&handle) {
            if !req.send_event(handle, event) {
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
                // FIXME set a timer to cancel the request if no messages are sent
                trace!("New request {}", handle);
                let pending = self.add_request(handle, sub_id, body, sender).unwrap();
                if !pending.send_event(handle, RequestExtEvent::Init()) {
                    trace!("Request sender dropped before Init");
                    self.remove_request(handle);
                }
                Ok(true)
            }
            NetworkerEvent::FinishRequest(handle) => {
                self.remove_request(handle);
                Ok(true)
            }
            NetworkerEvent::Dispatch(handle, target, timeout) => {
                // FIXME improve error handling
                trace!("Dispatch {} {:?}", handle, target);
                self.dispatch_request(handle, target, timeout).unwrap();
                Ok(true)
            }
            NetworkerEvent::CleanTimeout(handle, node_alias) => {
                self.clean_timeout(handle, node_alias).unwrap();
                Ok(true)
            }
            NetworkerEvent::ExtendTimeout(handle, node_alias, timeout) => {
                self.extend_timeout(handle, node_alias, timeout).unwrap();
                Ok(true)
            }
        }
    }

    fn clean_requests(&mut self) -> Result<(), String> {
        let checked = self
            .requests
            .iter()
            .flat_map(|(handle, req)| {
                if req.last_active.elapsed()
                    > Duration::from_secs(self.config.conn_active_timeout as u64)
                {
                    let active = self
                        .pool_connections
                        .get(&req.conn_id)
                        .map(|pc| !pc.has_active_timeouts(&req.sub_id))
                        .unwrap_or(false);
                    Some((*handle, active))
                } else {
                    None
                }
            })
            .collect::<Vec<(RequestHandle, bool)>>();
        for (handle, active) in checked {
            if active {
                trace!("Request still active: {}", handle);
                self.extend_active_request(handle)
            } else {
                trace!("Removing idle request: {}", handle);
                self.remove_request(handle)
            }
        }
        Ok(())
    }

    fn add_request(
        &mut self,
        handle: RequestHandle,
        sub_id: String,
        body: String,
        sender: Sender<RequestExtEvent>,
    ) -> LedgerResult<&mut PendingRequest> {
        let conn_id = self.get_active_connection(self.last_connection, Some(sub_id.clone()))?;
        let pending = PendingRequest {
            conn_id,
            send_index: 0,
            sender,
            sub_id,
            body,
            last_active: Instant::now(),
        };
        self.requests.insert(handle, pending);
        self.requests.get_mut(&handle).ok_or(err_msg(
            LedgerErrorKind::InvalidState,
            "Error adding request",
        ))
    }

    fn remove_request(&mut self, handle: RequestHandle) {
        if let Some(req) = self.requests.remove(&handle) {
            self.check_remove_connection(req.conn_id, Some(&req.sub_id));
        }
    }

    fn check_remove_connection(&mut self, handle: ZMQConnectionHandle, sub_id: Option<&str>) {
        if let Some(delete) = self.pool_connections.get_mut(&handle).and_then(|pc| {
            if let Some(sub_id) = sub_id {
                pc.clean_timeout(sub_id, None);
            }
            if pc.is_idle() {
                Some(&handle)
            } else {
                None
            }
        }) {
            trace!("Removing pool connection {}", delete);
            self.pool_connections.remove(delete);
            // DEBUG test active sockets
            let (_, poll_items) = self.get_poll_items();
            if poll_items.len() == 0 {
                trace!("No more sockets!");
            }
        }
    }

    fn extend_active_request(&mut self, handle: RequestHandle) {
        if let Some(req) = self.requests.get_mut(&handle) {
            req.last_active = Instant::now();
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
                let indices: Vec<usize> = match target {
                    RequestDispatchTarget::AllNodes => {
                        let idx = request.send_index;
                        (idx..(idx + self.nodes.len())).collect()
                    }
                    RequestDispatchTarget::AnyNodes(count) => {
                        let idx = request.send_index;
                        (idx..(idx + ::std::cmp::min(count, self.nodes.len()))).collect()
                    }
                    RequestDispatchTarget::SelectNodes(named) => named
                        .iter()
                        .flat_map(|name| {
                            conn.get_send_index(name).or_else(|| {
                                warn!("Cannot send to unknown node alias: {}", name);
                                None
                            })
                        })
                        .collect(),
                };
                for idx in indices {
                    let node_alias = conn.send_request(
                        request.sub_id.clone(),
                        request.body.clone(),
                        timeout,
                        idx,
                    )?;
                    if !request.send_event(
                        handle,
                        RequestExtEvent::Sent(node_alias, SystemTime::now(), request.send_index),
                    ) {
                        self.remove_request(handle);
                        break;
                    }
                    request.send_index += 1;
                    request.last_active = Instant::now();
                }
            } else {
                warn!("Pool connection expired for request: {}", handle);
                self.remove_request(handle)
            }
        } else {
            warn!("Unknown request ID for dispatch: {}", handle)
        }
        Ok(())
    }

    fn clean_timeout(&mut self, handle: RequestHandle, node_alias: String) -> LedgerResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                conn.clean_timeout(request.sub_id.as_str(), Some(node_alias))
            } else {
                warn!("Pool connection expired for clean timeout: {}", handle)
            }
        } else {
            warn!("Unknown request ID for clean timeout: {}", handle)
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
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
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
                        .map(|sub_id| !conn.seen_request(sub_id.as_str()))
                        .unwrap_or(true)
            });
        if conn.is_none() {
            let conn =
                ZMQConnection::new(self.randomize_remotes(), self.config.conn_active_timeout);
            trace!("Created new pool connection");
            let pc_id = ZMQConnectionHandle::next();
            self.pool_connections.insert(pc_id, conn);
            self.last_connection.replace(pc_id);
            Ok(pc_id)
        } else {
            Ok(conn_id.unwrap())
        }
    }

    fn randomize_remotes(&self) -> Vec<RemoteNode> {
        let mut result = vec![];
        let mut weights = self
            .weights
            .iter()
            .enumerate()
            .map(|(idx, w)| (idx, *w))
            .collect::<Vec<(usize, f32)>>();
        let mut rng = rand::thread_rng();
        for _ in 0..weights.len() {
            let idx = weights.choose_weighted(&mut rng, |item| item.1).unwrap().0;
            weights[idx].1 = 0f32;
            result.push(self.remotes[idx].clone());
        }
        result
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

    fn get_timeout(&self) -> (Option<(ZMQConnectionHandle, Option<(String, String)>)>, i64) {
        self.pool_connections
            .iter()
            .map(|(&handle, conn)| {
                let (last_req, timeout) = conn.get_timeout();
                (Some((handle, last_req)), timeout)
            })
            .min_by(|&(_, val1), &(_, val2)| val1.cmp(&val2))
            .unwrap_or((None, ::std::i64::MAX))
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
    timeouts: HashMap<(String, String), (Instant, bool)>,
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
            timeouts: HashMap::new(),
            req_cnt: 0,
            req_log: HashSet::new(),
            active_timeout,
        }
    }

    fn fetch_event(&self, index: usize, poll_item: &zmq::PollItem) -> Option<NodeEvent> {
        if let (&Some(ref s), rn) = (&self.sockets[index], &self.nodes[index]) {
            if poll_item.is_readable() {
                if let Ok(Ok(msg)) = s.recv_string(zmq::DONTWAIT) {
                    trace!("got reply {}", &msg);
                    match Message::from_raw_str(msg.as_str()) {
                        Ok(meta) => {
                            return Some(NodeEvent::Reply(
                                msg,
                                meta,
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

    fn get_timeout(&self) -> (Option<(String, String)>, i64) {
        let now = Instant::now();
        let (target, expiry) = {
            if let Some((&(ref req_id, ref node_alias), (timeout, _))) = self
                .timeouts
                .iter()
                .filter(|&(_, (_, seen))| !seen)
                .min_by(|&(_, (ref val1, _)), &(_, (ref val2, _))| val1.cmp(&val2))
            {
                (Some((req_id.to_string(), node_alias.to_string())), *timeout)
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
            expiry
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
            < Duration::from_secs(::std::cmp::max(self.active_timeout, 0) as u64);
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
        self.timeouts.insert(
            (req_id.clone(), name.clone()),
            (
                Instant::now() + Duration::from_secs(::std::cmp::max(timeout, 0) as u64),
                false,
            ),
        );
        self.req_cnt += 1;
        self.req_log.insert(req_id);
        trace!("send_request <<");
        Ok(name)
    }

    fn extend_timeout(&mut self, req_id: &str, node_alias: &str, extended_timeout: i64) {
        if let Some(timeout) = self
            .timeouts
            .get_mut(&(req_id.to_string(), node_alias.to_string()))
        {
            timeout.0 =
                Instant::now() + Duration::from_secs(::std::cmp::max(extended_timeout, 0) as u64);
            timeout.1 = false
        } else {
            debug!("late REQACK for req_id {}, node {}", req_id, node_alias);
        }
    }

    fn silence_timeout(&mut self, req_id: &str, node_alias: &str) {
        if let Some(timeout) = self
            .timeouts
            .get_mut(&(req_id.to_string(), node_alias.to_string()))
        {
            timeout.1 = true
        }
    }

    fn clean_timeout(&mut self, req_id: &str, node_alias: Option<String>) {
        match node_alias {
            Some(node_alias) => {
                self.timeouts.remove(&(req_id.to_string(), node_alias));
            }
            None => {
                let keys_to_remove: Vec<(String, String)> = self
                    .timeouts
                    .keys()
                    .cloned()
                    .filter(|&(ref req_id_timeout, _)| req_id == req_id_timeout)
                    .collect();
                keys_to_remove.iter().for_each(|key| {
                    self.timeouts.remove(key);
                });
            }
        }
    }

    fn has_active_timeouts(&self, req_id: &str) -> bool {
        self.timeouts
            .keys()
            .find(|&(req_id_timeout, _)| req_id_timeout == req_id)
            .is_some()
    }

    fn seen_request(&self, req_id: &str) -> bool {
        self.req_log.contains(req_id)
    }

    fn has_active_requests(&self) -> bool {
        !self.timeouts.is_empty()
    }

    fn is_idle(&self) -> bool {
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

#[derive(Clone, Eq, PartialEq, Hash)]
struct RemoteNode {
    pub name: String,
    pub public_key: Vec<u8>,
    pub zaddr: String,
    pub is_blacklisted: bool,
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

impl std::fmt::Debug for RemoteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pubkey = self.public_key.to_base58();
        write!(
            f,
            "RemoteNode {{ name: {}, public_key: {}, zaddr: {}, is_blacklisted: {:?} }}",
            self.name, pubkey, self.zaddr, self.is_blacklisted
        )
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
        String,     // message
        Message,    // parsed
        String,     // node alias
        SystemTime, // received time
    ),
    RequestTimeout(
        String, // req id
        String, // node alias
    ),
    Timeout(),
}

#[derive(Debug)]
struct PendingRequest {
    conn_id: ZMQConnectionHandle,
    send_index: usize,
    sender: Sender<RequestExtEvent>,
    sub_id: String,
    body: String,
    last_active: Instant,
}

impl PendingRequest {
    fn send_event(&mut self, handle: RequestHandle, event: RequestExtEvent) -> bool {
        if let Err(err) = self.sender.try_send(event) {
            // FIXME - return an enum that can be converted into log message
            if err.is_full() {
                trace!("Removing request: buffer full {}", handle);
            } else if err.is_disconnected() {
                trace!("Removing request: sender disconnected {}", handle);
            } else {
                trace!("Removing request: send error {} {:?}", handle, err);
            }
            return false;
        }
        return true;
    }
}
