use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use futures_channel::mpsc::UnboundedSender;

use zmq::PollItem;
use zmq::Socket as ZSocket;

use crate::common::error::prelude::*;
use crate::common::handle::ResourceHandle;
use crate::config::PoolConfig;
use crate::utils::{base58, base64};

use super::types::{Message, Verifiers};
use super::{Networker, NetworkerEvent, NetworkerFactory, RequestExtEvent, RequestHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ZMQSocketHandle(pub i64);

impl_sequence_handle!(ZMQSocketHandle, ZSC_COUNTER);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ZMQConnectionHandle(pub i64);

impl_sequence_handle!(ZMQConnectionHandle, ZCH_COUNTER);

/// ZeroMQ `NetworkerFactory` implementation
#[derive(Default)]
pub struct ZMQNetworkerFactory;

impl NetworkerFactory for ZMQNetworkerFactory {
    type Output = ZMQNetworker;
    fn make_networker(&self, config: PoolConfig, verifiers: &Verifiers) -> VdrResult<ZMQNetworker> {
        let remotes = _get_remotes(verifiers);
        let socket_handle = *ZMQSocketHandle::next();
        let (zmq_ctx, cmd_send, cmd_recv) =
            _create_pair_of_sockets(&format!("zmqnet_{}", socket_handle));
        let (evt_send, evt_recv) = mpsc::channel::<NetworkerEvent>();
        let worker = thread::spawn(move || {
            let mut zmq_thread = ZMQThread::new(config, zmq_ctx, cmd_recv, evt_recv, remotes);
            if let Err(err) = zmq_thread.work() {
                warn!("ZMQ worker exited with error: {}", err)
            } else {
                trace!("ZMQ worker exited");
            }
        });
        Ok(ZMQNetworker {
            cmd_send,
            evt_send,
            worker: Some(worker),
        })
    }
}

/// ZeroMQ `Networker` implementation
pub struct ZMQNetworker {
    cmd_send: zmq::Socket,
    evt_send: mpsc::Sender<NetworkerEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Networker for ZMQNetworker {
    fn send(&self, event: NetworkerEvent) -> VdrResult<()> {
        self.evt_send
            .send(event)
            .with_err_msg(VdrErrorKind::Resource, "Error sending networker event")?;
        // stop waiting on current sockets
        self.cmd_send
            .send("", 0)
            .with_err_msg(VdrErrorKind::Resource, "Error sending networker command")
    }
}

impl Drop for ZMQNetworker {
    fn drop(&mut self) {
        if self.cmd_send.send("exit", 0).is_err() {
            trace!("Networker command socket already closed")
        } else {
            trace!("Networker thread told to exit")
        }
        if let Some(worker) = self.worker.take() {
            debug!("Drop networker thread");
            worker.join().unwrap()
        }
    }
}

struct ZMQThread {
    config: PoolConfig,
    zmq_ctx: zmq::Context,
    cmd_recv: zmq::Socket,
    evt_recv: mpsc::Receiver<NetworkerEvent>,
    node_aliases: HashSet<String>,
    remotes: Vec<RemoteNode>,
    requests: BTreeMap<RequestHandle, PendingRequest>,
    last_connection: Option<ZMQConnectionHandle>,
    pool_connections: BTreeMap<ZMQConnectionHandle, ZMQConnection>,
}

impl ZMQThread {
    pub fn new(
        config: PoolConfig,
        zmq_ctx: zmq::Context,
        cmd_recv: zmq::Socket,
        evt_recv: mpsc::Receiver<NetworkerEvent>,
        remotes: Vec<RemoteNode>,
    ) -> Self {
        let node_aliases = HashSet::from_iter(remotes.iter().map(|r| r.name.clone()));
        ZMQThread {
            config,
            zmq_ctx,
            cmd_recv,
            evt_recv,
            node_aliases,
            remotes,
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
            // self.clean_requests()?
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
                    events.push((conn_id, ConnectionEvent::RequestTimeout(req_id, node_alias)));
                } else {
                    events.push((conn_id, ConnectionEvent::Timeout()));
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
        if !events.is_empty() {
            trace!("Got {} events", events.len());
            return Ok(PollResult::Events(events));
        } else {
            //print!(".")
        }
        if poll_items.len() == 1 {
            return Ok(PollResult::NoSockets);
        }
        Ok(PollResult::Default)
    }

    fn process_event(&mut self, conn_id: ZMQConnectionHandle, event: ConnectionEvent) {
        let (req_id, fwd) = match event {
            ConnectionEvent::Reply(message, meta, node_alias, time) => {
                let req_id = meta.request_id().unwrap_or_default();
                if let Some(conn) = self.pool_connections.get_mut(&conn_id) {
                    conn.clean_idle_timeout(&req_id);
                }
                (
                    req_id,
                    RequestExtEvent::Received(node_alias, message, meta, time),
                )
            }
            ConnectionEvent::RequestTimeout(req_id, node_alias) => {
                if node_alias.is_empty() {
                    if let Some(handle) = self.select_request(conn_id, &req_id) {
                        trace!("Remove idle {}", handle);
                        self.remove_request(handle);
                    }
                    return;
                }
                if let Some(conn) = self.pool_connections.get_mut(&conn_id) {
                    conn.clean_timeout(&req_id, Some(node_alias.clone()));
                }
                (req_id, RequestExtEvent::Timeout(node_alias))
            }
            ConnectionEvent::Timeout() => {
                self.check_remove_connection(conn_id, None);
                return;
            }
        };
        if let Some(handle) = self.select_request(conn_id, &req_id) {
            self.process_reply(handle, fwd)
        } else {
            trace!("Unknown request ID: {}", req_id)
        }
    }

    fn select_request(&self, conn_id: ZMQConnectionHandle, sub_id: &str) -> Option<RequestHandle> {
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
            if !req.send_event(event) {
                trace!("Removing, sender disconnected {}", handle);
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
                if !pending.send_event(RequestExtEvent::Init) {
                    trace!("Removing, sender dropped before Init {}", handle);
                    self.remove_request(handle);
                }
                Ok(true)
            }
            NetworkerEvent::FinishRequest(handle) => {
                trace!("Removing, finished {}", handle);
                self.remove_request(handle);
                Ok(true)
            }
            NetworkerEvent::Dispatch(handle, node_aliases, timeout) => {
                // FIXME improve error handling
                trace!("Dispatch {} {:?}", handle, node_aliases);
                self.dispatch_request(handle, node_aliases, timeout)
                    .unwrap();
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

    fn add_request(
        &mut self,
        handle: RequestHandle,
        sub_id: String,
        body: String,
        sender: UnboundedSender<RequestExtEvent>,
    ) -> VdrResult<&mut PendingRequest> {
        let conn_id = self.get_active_connection(self.last_connection, sub_id.clone())?;
        let pending = PendingRequest {
            conn_id,
            sender,
            sub_id,
            body,
        };
        self.requests.insert(handle, pending);
        self.requests
            .get_mut(&handle)
            .ok_or_else(|| input_err("Error adding request"))
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
            if poll_items.is_empty() {
                trace!("No more sockets!");
            }
        }
    }

    fn dispatch_request(
        &mut self,
        handle: RequestHandle,
        node_aliases: Vec<String>,
        timeout: i64,
    ) -> VdrResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                for node_alias in node_aliases {
                    if !self.node_aliases.contains(&node_alias) {
                        warn!("Cannot send to unknown node alias: {}", node_alias);
                        continue;
                    }
                    conn.send_request(
                        request.sub_id.clone(),
                        request.body.clone(),
                        node_alias.clone(),
                        timeout,
                    )?;
                    if !request.send_event(RequestExtEvent::Sent(node_alias, SystemTime::now())) {
                        trace!("Removing, sender disconnected {}", handle);
                        self.remove_request(handle);
                        break;
                    }
                }
            } else {
                warn!("Removing, pool connection expired {}", handle);
                self.remove_request(handle)
            }
        } else {
            debug!("Unknown request ID for dispatch: {}", handle)
        }
        Ok(())
    }

    fn clean_timeout(&mut self, handle: RequestHandle, node_alias: String) -> VdrResult<()> {
        if let Some(request) = self.requests.get(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                conn.clean_timeout(request.sub_id.as_str(), Some(node_alias))
            } else {
                debug!("Pool connection expired for clean timeout: {}", handle)
            }
        } else {
            debug!("Unknown request ID for clean timeout: {}", handle)
        }
        Ok(())
    }

    fn extend_timeout(
        &mut self,
        handle: RequestHandle,
        node_alias: String,
        timeout: i64,
    ) -> VdrResult<()> {
        if let Some(request) = self.requests.get_mut(&handle) {
            if let Some(conn) = self.pool_connections.get_mut(&request.conn_id) {
                conn.extend_timeout(request.sub_id.as_str(), node_alias.as_str(), timeout)
            } else {
                debug!("Pool connection expired for extend timeout: {}", handle)
            }
        } else {
            debug!("Unknown request ID for extend timeout: {}", handle)
        }
        Ok(())
    }

    fn get_active_connection(
        &mut self,
        conn_id: Option<ZMQConnectionHandle>,
        sub_id: String,
    ) -> VdrResult<ZMQConnectionHandle> {
        let req_limit = self.config.conn_request_limit;
        let conn = conn_id
            .and_then(|conn_id| self.pool_connections.get_mut(&conn_id))
            .filter(|conn| {
                conn.is_active() && conn.req_cnt < req_limit && !conn.seen_request(sub_id.as_str())
            });
        if let Some(conn) = conn {
            conn.init_request(sub_id);
            Ok(conn_id.unwrap())
        } else {
            let mut conn = ZMQConnection::new(
                self.zmq_ctx.clone(),
                self.remotes.clone(),
                self.config.conn_active_timeout,
                self.config.ack_timeout,
                self.config.socks_proxy.clone(),
            );
            trace!("Created new pool connection");
            conn.init_request(sub_id);
            let pc_id = ZMQConnectionHandle::next();
            self.pool_connections.insert(pc_id, conn);
            self.last_connection.replace(pc_id);
            debug!("New {}", pc_id);
            Ok(pc_id)
        }
    }

    fn fetch_events_into(
        &self,
        conn_idx: &[(ZMQConnectionHandle, usize)],
        poll_items: &[PollItem],
        events: &mut Vec<(ZMQConnectionHandle, ConnectionEvent)>,
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

    #[allow(clippy::type_complexity)]
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
    remotes: Vec<RemoteNode>,
    sockets: Vec<Option<ZSocket>>,
    ctx: zmq::Context,
    key_pair: zmq::CurveKeyPair,
    idle_timeouts: HashMap<String, Instant>,
    socket_timeouts: HashMap<(String, String), Instant>,
    time_created: Instant,
    req_cnt: usize,
    req_log: HashSet<String>,
    active_timeout: i64,
    idle_timeout: i64,
    socks_proxy: Option<String>,
}

impl ZMQConnection {
    fn new(
        zmq_ctx: zmq::Context,
        remotes: Vec<RemoteNode>,
        active_timeout: i64,
        idle_timeout: i64,
        socks_proxy: Option<String>,
    ) -> Self {
        trace!("ZMQConnection::new: from remotes {:?}", remotes);

        let sockets = (0..remotes.len())
            .map(|_| None)
            .collect::<Vec<Option<ZSocket>>>();

        Self {
            remotes,
            sockets,
            ctx: zmq_ctx,
            key_pair: zmq::CurveKeyPair::new().expect("FIXME"),
            time_created: Instant::now(),
            idle_timeouts: HashMap::new(),
            socket_timeouts: HashMap::new(),
            req_cnt: 0,
            req_log: HashSet::new(),
            active_timeout,
            idle_timeout,
            socks_proxy,
        }
    }

    fn fetch_event(&self, index: usize, poll_item: &zmq::PollItem) -> Option<ConnectionEvent> {
        if let (Some(ref s), rn) = (&self.sockets[index], &self.remotes[index]) {
            if poll_item.is_readable() {
                if let Ok(Ok(msg)) = s.recv_string(zmq::DONTWAIT) {
                    trace!("Socket reply {} {}", &rn.name, &msg);
                    match Message::from_raw_str(msg.as_str()) {
                        Ok(meta) => {
                            return Some(ConnectionEvent::Reply(
                                msg,
                                meta,
                                rn.name.clone(),
                                SystemTime::now(),
                            ));
                        }
                        Err(err) => debug!("Error parsing received message: {:?}", err),
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
            let min_idle = self
                .idle_timeouts
                .iter()
                .min_by(|&(_, inst1), &(_, inst2)| inst1.cmp(inst2))
                .map(|(req_id, inst)| ((req_id.as_str(), ""), inst));
            if let Some(((req_id, node_alias), timeout)) = self
                .socket_timeouts
                .iter()
                .map(|((req_id, alias), inst)| ((req_id.as_str(), alias.as_str()), inst))
                .chain(min_idle)
                .min_by(|(_, ref val1), (_, ref val2)| val1.cmp(val2))
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

    fn send_request(
        &mut self,
        req_id: String,
        msg: String,
        node_alias: String,
        timeout: i64,
    ) -> VdrResult<()> {
        trace!("send_request >> req_id: {} node: {}", req_id, node_alias);
        let node_index = self.remotes.iter().position(|node| node.name == node_alias);
        if let Some(node_index) = node_index {
            let s = self._get_socket(node_index)?;
            s.send(&msg, zmq::DONTWAIT)?;
        } else {
            warn!("Cannot send to unknown node alias: {}", node_alias);
            return Ok(());
        }
        if self.idle_timeouts.contains_key(&req_id) {
            // will only be present if this request has received no responses
            self.set_idle_timeout(req_id.clone());
        }
        self.add_timeout(req_id, node_alias, timeout);
        trace!("send_request <<");
        Ok(())
    }

    fn add_timeout(&mut self, req_id: String, alias: String, timeout: i64) {
        self.socket_timeouts.insert(
            (req_id, alias),
            Instant::now() + Duration::from_secs(::std::cmp::max(timeout, 0) as u64),
        );
    }

    fn extend_timeout(&mut self, req_id: &str, node_alias: &str, extended_timeout: i64) {
        if let Some(timeout) = self
            .socket_timeouts
            .get_mut(&(req_id.to_string(), node_alias.to_string()))
        {
            *timeout =
                Instant::now() + Duration::from_secs(::std::cmp::max(extended_timeout, 0) as u64)
        } else {
            debug!("late REQACK for req_id {}, node {}", req_id, node_alias);
        }
    }

    fn clean_timeout(&mut self, req_id: &str, node_alias: Option<String>) {
        match node_alias {
            Some(node_alias) => {
                if self
                    .socket_timeouts
                    .remove(&(req_id.to_string(), node_alias))
                    .is_some()
                    && !self
                        .socket_timeouts
                        .keys()
                        .any(|(ref req_id_timeout, _)| req_id == req_id_timeout)
                {
                    self.set_idle_timeout(req_id.to_string())
                }
            }
            None => {
                let keys_to_remove: Vec<(String, String)> = self
                    .socket_timeouts
                    .keys()
                    .cloned()
                    .filter(|(ref req_id_timeout, _)| req_id == req_id_timeout)
                    .collect();
                keys_to_remove.iter().for_each(|key| {
                    self.socket_timeouts.remove(key);
                });
                self.idle_timeouts.remove(req_id);
            }
        }
    }

    fn set_idle_timeout(&mut self, req_id: String) {
        self.idle_timeouts.insert(
            req_id,
            Instant::now() + Duration::from_secs(::std::cmp::max(self.idle_timeout, 0) as u64),
        );
    }

    fn clean_idle_timeout(&mut self, req_id: &str) {
        self.idle_timeouts.remove(req_id);
    }

    fn init_request(&mut self, req_id: String) {
        self.req_cnt += 1;
        self.req_log.insert(req_id.clone());
        self.set_idle_timeout(req_id);
    }

    fn seen_request(&self, req_id: &str) -> bool {
        self.req_log.contains(req_id)
    }

    fn has_active_requests(&self) -> bool {
        !(self.socket_timeouts.is_empty() && self.idle_timeouts.is_empty())
    }

    fn is_idle(&self) -> bool {
        !self.is_active() && !self.has_active_requests()
    }

    fn _get_socket(&mut self, idx: usize) -> VdrResult<&ZSocket> {
        if self.sockets[idx].is_none() {
            debug!("Open new socket for node {}", &self.remotes[idx].name);
            let s: ZSocket =
                self.remotes[idx].connect(&self.ctx, &self.key_pair, self.socks_proxy.clone())?;
            self.sockets[idx] = Some(s)
        }
        Ok(self.sockets[idx].as_ref().unwrap())
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct RemoteNode {
    pub name: String,
    pub enc_key: Vec<u8>,
    pub zaddr: String,
    pub is_blacklisted: bool,
}

impl RemoteNode {
    fn connect(
        &self,
        ctx: &zmq::Context,
        key_pair: &zmq::CurveKeyPair,
        socks_proxy: Option<String>,
    ) -> VdrResult<ZSocket> {
        let s = ctx.socket(zmq::SocketType::DEALER)?;
        s.set_identity(base64::encode(key_pair.public_key).as_bytes())?;
        s.set_curve_secretkey(&key_pair.secret_key)?;
        s.set_curve_publickey(&key_pair.public_key)?;
        s.set_curve_serverkey(
            zmq::z85_encode(self.enc_key.as_slice())
                .with_input_err("Can't encode server key as z85")? // FIXME: review kind
                .as_bytes(),
        )?;
        s.set_linger(0)?; //TODO set correct timeout
        if let Some(socks_proxy) = socks_proxy {
            let proxy = socks_proxy;
            debug!("Use socks proxy: {}", proxy);
            let result = s.set_socks_proxy(Some(&proxy));
            if result.is_err() {
                error!("socks error: {}", result.unwrap_err())
            }
        } else {
            debug!("Socks proxy is not configured");
        }
        s.connect(&self.zaddr)?;
        Ok(s)
    }
}

impl std::fmt::Debug for RemoteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pubkey = base58::encode(&self.enc_key);
        write!(
            f,
            "RemoteNode {{ name: {}, public_key: {}, zaddr: {}, is_blacklisted: {:?} }}",
            self.name, pubkey, self.zaddr, self.is_blacklisted
        )
    }
}

fn _create_pair_of_sockets(addr: &str) -> (zmq::Context, zmq::Socket, zmq::Socket) {
    let zmq_ctx = zmq::Context::new();
    let send_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();
    let recv_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();

    let inproc_sock_name: String = format!("inproc://{}", addr);
    recv_cmd_sock.bind(inproc_sock_name.as_str()).unwrap();
    send_cmd_sock.connect(inproc_sock_name.as_str()).unwrap();
    (zmq_ctx, send_cmd_sock, recv_cmd_sock)
}

fn _get_remotes(verifiers: &Verifiers) -> Vec<RemoteNode> {
    verifiers
        .iter()
        .map(|(alias, info)| RemoteNode {
            name: alias.clone(),
            enc_key: info.enc_key.clone(),
            zaddr: info.client_addr.clone(),
            is_blacklisted: false,
        })
        .collect()
}

#[derive(Debug)]
enum PollResult {
    Default,
    Events(Vec<(ZMQConnectionHandle, ConnectionEvent)>),
    NoSockets,
    Exit,
}

#[derive(Debug)]
enum ConnectionEvent {
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
    sender: UnboundedSender<RequestExtEvent>,
    sub_id: String,
    body: String,
}

impl PendingRequest {
    fn send_event(&mut self, event: RequestExtEvent) -> bool {
        self.sender.unbounded_send(event).is_ok()
    }
}
