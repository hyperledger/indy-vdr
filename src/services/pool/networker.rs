use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::FromBase58;
use crate::utils::crypto;
use crate::utils::error::prelude::*;

use super::events::PoolEvent;
use super::merkle_tree_factory::build_node_state_from_json;
use super::request_handler::{
    HandlerEvent, HandlerUpdate, PoolRequest, PoolRequestHandler, PoolRequestSubscribe,
    PoolRequestTarget, PoolRequestTimeout,
};
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

new_handle_type!(ZMQConnectionHandle, PHC_COUNTER);

new_handle_type!(NetworkerHandle, NH_COUNTER);

#[derive(Debug)]
pub enum NetworkerEvent {
    SendOneRequest(
        String, //msg
        String, //req_id
        i64,    //timeout
    ),
    SendAllRequest(
        String,              //msg
        String,              //req_id
        i64,                 //timeout
        Option<Vec<String>>, //nodes
    ),
    Resend(
        String, //req_id
        i64,    //timeout
    ),
    /*NodesStateUpdated(
        Vec<RemoteNode>,
        Option<Vec<String>>, // preferred order
    ),*/
    ExtendTimeout(
        String, //req_id
        String, //node_alias
        i64,    //timeout
    ),
    CleanTimeout(
        String,         //req_id
        Option<String>, //node_alias
    ),
    Timeout,
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

enum PollResult {
    Default,
    Events(Vec<NodeEvent>),
    NoSockets,
    Exit,
}

struct PendingRequest {
    msg: String,
    sent: Vec<(u32, SystemTime, ZMQConnectionHandle)>,
    subscribe: PoolRequestSubscribe,
    handler: RefCell<Box<dyn PoolRequestHandler>>,
}

pub trait Networker: Sized {
    fn new(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
        sender: mpsc::Sender<PoolEvent>,
    ) -> LedgerResult<Self>;
    fn get_id(&self) -> NetworkerHandle;
    fn add_request(&mut self, request: PoolRequest) -> LedgerResult<()>;
}

pub struct ZMQNetworker {
    id: NetworkerHandle,
    cmd_send: zmq::Socket,
    req_send: mpsc::Sender<PoolRequest>,
    sender: mpsc::Sender<PoolEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Networker for ZMQNetworker {
    fn new(
        config: PoolConfig,
        transactions: Vec<String>,
        preferred_nodes: Vec<String>,
        sender: mpsc::Sender<PoolEvent>,
    ) -> LedgerResult<Self> {
        let id = NetworkerHandle::next();
        let (nodes, remotes) = _get_nodes_and_remotes(transactions, config.protocol_version)?;
        let (req_send, req_recv) = mpsc::channel::<PoolRequest>();
        let (cmd_send, cmd_recv) = _create_pair_of_sockets(&format!("zmqnet_{}", id));
        let mut result = ZMQNetworker {
            id,
            cmd_send,
            req_send,
            sender,
            worker: None,
        };
        result.start(config, cmd_recv, req_recv, nodes, remotes, preferred_nodes);
        Ok(result)
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

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
            worker.thread().unpark();
            info!("Drop networker thread");
            worker.join().unwrap()
        }
    }
}

impl ZMQNetworker {
    fn start(
        &mut self,
        config: PoolConfig,
        cmd_recv: zmq::Socket,
        req_recv: mpsc::Receiver<PoolRequest>,
        nodes: Nodes,
        remotes: Vec<RemoteNode>,
        preferred_nodes: Vec<String>,
    ) {
        let sender = self.sender.clone();
        let mut weights: Vec<(usize, f32)> = (0..remotes.len()).map(|idx| (idx, 1.0)).collect();
        for name in preferred_nodes {
            if let Some(index) = remotes.iter().position(|node| node.name == name) {
                weights[index] = (index, 2.0);
            }
        }
        // FIXME adjust weights up for preferred
        self.worker.replace(thread::spawn(move || {
            let mut zmq_thread =
                ZMQThread::new(config, cmd_recv, req_recv, sender, nodes, remotes, weights);
            zmq_thread.work().map_err(map_err_err!());
            // FIXME send pool event when networker exits
        }));
    }
}

// FIXME - impl Drop for ZMQNetworker

struct ZMQThread {
    config: PoolConfig,
    cmd_recv: zmq::Socket,
    req_recv: mpsc::Receiver<PoolRequest>,
    sender: mpsc::Sender<PoolEvent>,
    nodes: Nodes,
    remotes: Vec<RemoteNode>,
    weights: Vec<(usize, f32)>,
    requests: HashMap<String, PendingRequest>,
    last_connection: Option<ZMQConnectionHandle>,
    pool_connections: BTreeMap<ZMQConnectionHandle, ZMQConnection>,
}

impl ZMQThread {
    pub fn new(
        config: PoolConfig,
        cmd_recv: zmq::Socket,
        req_recv: mpsc::Receiver<PoolRequest>,
        sender: mpsc::Sender<PoolEvent>,
        nodes: Nodes,
        remotes: Vec<RemoteNode>,
        weights: Vec<(usize, f32)>,
    ) -> Self {
        // FIXME adjust weights up for preferred nodes
        ZMQThread {
            config,
            cmd_recv,
            req_recv,
            sender,
            nodes,
            remotes,
            weights,
            requests: HashMap::new(),
            last_connection: None,
            pool_connections: BTreeMap::new(),
        }
    }

    pub fn work(&mut self) -> Result<(), String> {
        loop {
            while self.receive_request(false)? {}
            match self.poll_events() {
                PollResult::NoSockets => {
                    // wait until a request is received
                    thread::park()
                }
                PollResult::Events(events) => {
                    for event in events {
                        self.process_event(event)
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

    fn poll_events(&mut self) -> PollResult {
        let mut poll_items;
        poll_items = self.get_poll_items();
        let (last_req, timeout) = self.get_timeout();
        let mut events = vec![];
        poll_items.push(self.cmd_recv.as_poll_item(zmq::POLLIN));
        let poll_res = zmq::poll(&mut poll_items, ::std::cmp::max(timeout, 0))
            .map_err(map_err_err!())
            .map_err(|_| unimplemented!() /* FIXME */)
            .unwrap();
        //            trace!("poll_res: {:?}", poll_res);
        if poll_res == 0 {
            if let Some((req_id, node_alias)) = last_req {
                events.push(NodeEvent::Timeout(req_id, node_alias));
            }
        } else {
            self.fetch_events_into(poll_items.as_slice(), &mut events);
        }
        if poll_items[poll_items.len() - 1].is_readable() {
            if let Ok(Ok(msg)) = self.cmd_recv.recv_string(zmq::DONTWAIT) {
                if msg == "exit" {
                    return PollResult::Exit;
                }
            } else {
                // command socket failed
                return PollResult::Exit;
            }
        }
        if events.len() > 0 {
            trace!("Got {} events", events.len());
            return PollResult::Events(events);
        } else {
            print!(".")
        }
        if poll_items.len() == 1 {
            return PollResult::NoSockets;
        }
        return PollResult::Default;
    }

    fn process_event(&mut self, event: NodeEvent) {
        match event {
            NodeEvent::Reply(req_id, message, node_alias, time) => match &message {
                Message::LedgerStatus(_) | Message::ConsistencyProof(_) => {
                    let reqs = self.select_requests(PoolRequestSubscribe::Status);
                    for req_id in reqs {
                        self.process_reply(
                            req_id.clone(),
                            HandlerEvent::Received(node_alias.clone(), &message, time),
                        )
                    }
                }
                Message::CatchupReq(_) => {
                    warn!("Ignoring catchup request");
                }
                Message::CatchupRep(_) => {
                    let reqs = self.select_requests(PoolRequestSubscribe::Catchup);
                    for req_id in reqs {
                        self.process_reply(
                            req_id.clone(),
                            HandlerEvent::Received(node_alias.clone(), &message, time),
                        )
                    }
                }
                _ => {
                    trace!("Unhandled message {:?}", message);
                }
            },
            NodeEvent::Timeout(req_id, node_alias) => {
                self.process_reply(req_id, HandlerEvent::Timeout(node_alias.clone()))
            }
        }
    }

    fn select_requests(&self, subscribed: PoolRequestSubscribe) -> Vec<String> {
        self.requests
            .iter()
            .filter_map(|(req_id, req)| {
                if req.subscribe == subscribed {
                    Some(req_id)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    fn send_update(&self, event: PoolEvent) {
        if let Err(_) = self.sender.send(event) {
            trace!("Lost pool update sender")
        }
    }

    fn clean_timeout(&mut self, req_id: &String, node_alias: Option<String>) {
        let delete: Vec<ZMQConnectionHandle> = self
            .pool_connections
            .iter()
            .filter_map(|(id, pc)| {
                pc.clean_timeout(&req_id, node_alias.clone());
                if pc.is_orphaned() {
                    Some(id)
                } else {
                    None
                }
            })
            .cloned()
            .collect();
        for idx in delete {
            trace!("removing pool connection {}", idx);
            self.pool_connections.remove(&idx);
        }
    }

    fn remove_request(&mut self, req_id: String) {
        if let Some(_) = self.requests.get(&req_id) {
            self.requests.remove(&req_id);
            self.clean_timeout(&req_id, None)
        }
    }

    fn process_reply(&mut self, req_id: String, event: HandlerEvent) {
        if let Some(req) = self.requests.get_mut(&req_id) {
            let update = req.handler.get_mut().process_event(req_id.clone(), event);
            trace!("got update {:?}", update);
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
            }
        } else {
            trace!("Request ID not found: {}", req_id);
        }
    }

    fn receive_request(&mut self, block: bool) -> Result<bool, String> {
        let request = if block {
            self.req_recv.recv().map_err(|err| err.to_string())?
        } else {
            match self.req_recv.try_recv() {
                Ok(request) => request,
                Err(mpsc::TryRecvError::Empty) => return Ok(false),
                Err(err) => return Err(err.to_string()),
            }
        };
        let req_id = request.req_id.clone();
        let result = self.add_request(request);
        self.sender
            .send(PoolEvent::SubmitAck(req_id.clone(), result))
            .map_err(|err| err.to_string())?;
        Ok(true)
    }

    fn add_request(&mut self, request: PoolRequest) -> LedgerResult<()> {
        let req_id = request.req_id.clone();
        if self.requests.contains_key(&req_id) {
            // FIXME send back duplicate request PoolEvent
            trace!("request with duplicate ID ignored");
            Ok(())
        } else {
            self.dispatch(request)
        }
    }

    fn get_active_connection(
        &mut self,
        conn_id: Option<ZMQConnectionHandle>,
    ) -> ZMQConnectionHandle {
        let conn = conn_id
            .and_then(|conn_id| self.pool_connections.get(&conn_id))
            .filter(|conn| conn.is_active() && conn.req_cnt < self.config.conn_request_limit);
        if conn.is_none() {
            let conn = ZMQConnection::new(self.remotes.clone(), self.config.conn_active_timeout);
            trace!("Created new pool connection");
            let pc_id = ZMQConnectionHandle::next();
            self.pool_connections.insert(pc_id, conn);
            self.last_connection.replace(pc_id);
            pc_id
        } else {
            self.last_connection.unwrap()
        }
    }

    fn dispatch(&mut self, request: PoolRequest) -> LedgerResult<()> {
        let PoolRequest {
            req_id,
            req_json,
            handler,
        } = request;
        let (init_target, subscribe, init_timeout) = handler.borrow().get_target();
        let pending = PendingRequest {
            msg: req_json.clone(),
            sent: vec![],
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
        )
    }

    fn select_timeout(&self, timeout: PoolRequestTimeout) -> i64 {
        match timeout {
            PoolRequestTimeout::Default => self.config.reply_timeout,
            PoolRequestTimeout::Ack => self.config.ack_timeout,
            PoolRequestTimeout::Seconds(n) => n,
        }
    }

    fn select_targets(&self, target: PoolRequestTarget) -> LedgerResult<Vec<usize>> {
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
    }

    fn send_request(
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
    }

    fn fetch_events_into(&self, poll_items: &[PollItem], events: &mut Vec<NodeEvent>) {
        let mut cnt = 0;
        events.extend(
            self.pool_connections
                .values()
                .map(|pc| {
                    let ocnt = cnt;
                    cnt += pc.sockets.iter().filter(|s| s.is_some()).count();
                    pc.fetch_events(&poll_items[ocnt..cnt])
                })
                .flat_map(|v| v.into_iter()),
        );
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

    fn get_timeout(&self) -> (Option<(String, String)>, i64) {
        self.pool_connections
            .values()
            .map(ZMQConnection::get_timeout)
            .min_by(|&(_, val1), &(_, val2)| val1.cmp(&val2))
            .unwrap_or((None, 0))
    }

    fn get_poll_items(&self) -> Vec<PollItem> {
        self.pool_connections
            .values()
            .flat_map(ZMQConnection::get_poll_items)
            .collect()
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

    fn fetch_events(&self, poll_items: &[zmq::PollItem]) -> Vec<NodeEvent> {
        let mut vec = Vec::new();
        let mut pi_idx = 0;
        let len = self.nodes.len();
        assert_eq!(len, self.sockets.len());
        for i in 0..len {
            if let (&Some(ref s), rn) = (&self.sockets[i], &self.nodes[i]) {
                if poll_items[pi_idx].is_readable() {
                    if let Ok(Ok(msg)) = s.recv_string(zmq::DONTWAIT) {
                        match Message::from_raw_str(msg.as_str()) {
                            Ok(message) => vec.push(NodeEvent::Reply(
                                message.request_id().unwrap_or("".to_string()),
                                message,
                                rn.name.clone(),
                                SystemTime::now(),
                            )),
                            Err(err) => error!("Error parsing received message: {:?}", err),
                        }
                    }
                }
                pi_idx += 1;
            }
        }
        vec
    }

    fn get_poll_items(&self) -> Vec<PollItem> {
        self.sockets
            .iter()
            .flat_map(|zs: &Option<ZSocket>| zs.as_ref().map(|zs| zs.as_poll_item(zmq::POLLIN)))
            .collect()
    }

    fn get_timeout(&self) -> (Option<(String, String)>, i64) {
        let now = Instant::now();
        let (target, timeout) = {
            if let Some((&(ref req_id, ref node_alias), timeout)) = self
                .timeouts
                .borrow()
                .iter()
                .min_by(|&(_, ref val1), &(_, ref val2)| val1.cmp(&val2))
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
        _sender: mpsc::Sender<PoolEvent>,
    ) -> LedgerResult<Self> {
        Ok(Self {
            id: NetworkerHandle::next(),
            events: Vec::new(),
        })
    }

    fn get_id(&self) -> NetworkerHandle {
        self.id
    }

    fn add_request(&mut self, _request: PoolRequest) -> LedgerResult<()> {
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
