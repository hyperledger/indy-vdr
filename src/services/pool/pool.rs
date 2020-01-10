// use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use super::events::PoolEvent;
use super::merkle_tree_factory::build_tree;
use super::networker::{Networker, NetworkerHandle, ZMQNetworker};
use super::request_handler::ledger_status_request;
use super::types::{CommandHandle, PoolConfig};
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

struct PoolThread<T: Networker> {
    config: PoolConfig,
    networker: Option<T>,
    evt_recv: Receiver<PoolEvent>,
    evt_send: Sender<PoolEvent>,
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
        let merkle_tree = build_tree(&txns)?;
        let mut networker = T::new(self.config, txns, vec![], self.evt_send.clone())?;
        let req = ledger_status_request(merkle_tree, self.config)?;
        // FIXME set up link between cmd_id and request
        networker.add_request(req)?;
        self.networker = Some(networker);
        Ok(())
    }

    fn handle_event(&mut self, event: PoolEvent) -> bool {
        match event {
            PoolEvent::Connect(_cmd_id, txns) => {
                // FIXME handle error, send update to listener
                self.connect(txns).unwrap();
            }
            PoolEvent::SubmitAck(req_id, result) => match result {
                Ok(_) => info!("Request dispatched {}", req_id),
                Err(err) => warn!("Request dispatch failed {} {}", req_id, err.to_string()),
            },
            PoolEvent::StatusSynced(_req_id, timing) => {
                info!("Synced! {:?}", timing)
                // FIXME send update to listener
            }
            PoolEvent::CatchupTargetFound(_req_id, mt_root, mt_size, timing) => {
                info!(
                    "Catchup target found {} {} {:?}",
                    mt_root.to_base58(),
                    mt_size,
                    timing
                )
                // FIXME start catchup
            }
            PoolEvent::CatchupTargetNotFound(req_id, err, _timing) => {
                print!("Catchup target not found {} {:?}", req_id, err)
                // FIXME send update to listener
            }
            PoolEvent::Exit() => {
                // networker(s) automatically dropped
                return true;
            }
            _ => trace!("Unhandled event {:?}", event),
        };
        false
    }
}

/*

struct PoolSM<T: Networker, R: RequestHandler<T>> {
    id: PoolHandle,
    config: PoolConfig,
    state: PoolState<T, R>,
}

/// Transitions of pool state
/// Inactive -> SyncCatchup, Active, Closed
/// Active -> SyncCatchup, Inactive, Closed
/// SyncCatchup -> Active, Inactive, Closed
/// Closed -> Closed
enum PoolState<T: Networker, R: RequestHandler<T>> {
    Inactive(InactiveState<T>),
    SyncCatchup(SyncCatchupState<T, R>),
    Active(ActiveState<T, R>),
    Closed(ClosedState),
}

struct InactiveState<T: Networker> {
    networker: Rc<RefCell<T>>,
}

struct ActiveState<T: Networker, R: RequestHandler<T>> {
    networker: Rc<RefCell<T>>,
    request_handlers: HashMap<String, R>,
    merkle_tree: MerkleTree,
    nodes: Nodes,
    sync_request: Option<(CommandHandle, R)>,
}

struct SyncCatchupState<T: Networker, R: RequestHandler<T>> {
    networker: Rc<RefCell<T>>,
    request_handler: R,
    cmd_id: CommandHandle,
}

struct ClosedState {}

impl<T: Networker, R: RequestHandler<T>> PoolSM<T, R> {
    pub fn new(id: PoolHandle, config: PoolConfig, networker: Rc<RefCell<T>>) -> PoolSM<T, R> {
        PoolSM {
            id,
            config: config,
            state: PoolState::Inactive(InactiveState { networker }),
        }
    }

    pub fn step(id: PoolHandle, config: PoolConfig, state: PoolState<T, R>) -> Self {
        PoolSM { id, config, state }
    }
}

// transitions from Inactive

impl<T: Networker> From<InactiveState<T>> for ClosedState {
    fn from(_state: InactiveState<T>) -> ClosedState {
        trace!("PoolSM: from init to closed");
        ClosedState {}
    }
}

impl<T: Networker, R: RequestHandler<T>> From<(InactiveState<T>, MerkleTree, Nodes)>
    for ActiveState<T, R>
{
    fn from(
        (state, merkle_tree, nodes): (InactiveState<T>, MerkleTree, Nodes),
    ) -> ActiveState<T, R> {
        trace!("PoolSM: from init to active");
        ActiveState {
            networker: state.networker,
            request_handlers: HashMap::new(),
            merkle_tree,
            nodes,
            sync_request: None,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<(InactiveState<T>, R, CommandHandle)>
    for SyncCatchupState<T, R>
{
    fn from((state, request_handler, cmd_id): (InactiveState<T>, R, CommandHandle)) -> Self {
        trace!("PoolSM: from init to sync catchup");
        SyncCatchupState {
            networker: state.networker,
            request_handler,
            cmd_id: cmd_id,
        }
    }
}

// transitions from SyncCatchup

impl<T: Networker, R: RequestHandler<T>> From<SyncCatchupState<T, R>> for InactiveState<T> {
    fn from(mut state: SyncCatchupState<T, R>) -> Self {
        trace!("PoolSM: from sync catchup to inactive");
        state
            .request_handler
            .process_event(Some(RequestEvent::Terminate));
        InactiveState {
            networker: state.networker,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<(SyncCatchupState<T, R>, MerkleTree, Nodes)>
    for ActiveState<T, R>
{
    fn from((state, merkle_tree, nodes): (SyncCatchupState<T, R>, MerkleTree, Nodes)) -> Self {
        ActiveState {
            networker: state.networker,
            request_handlers: HashMap::new(),
            nodes,
            merkle_tree,
            sync_request: None,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<SyncCatchupState<T, R>> for ClosedState {
    fn from(mut state: SyncCatchupState<T, R>) -> Self {
        trace!("PoolSM: from sync catchup to closed");
        state
            .request_handler
            .process_event(Some(RequestEvent::Terminate));
        ClosedState {}
    }
}

// transitions from Active

impl<T: Networker, R: RequestHandler<T>> From<ActiveState<T, R>> for InactiveState<T> {
    fn from(state: ActiveState<T, R>) -> Self {
        trace!("PoolSM: from active to Inactive");
        // FIXME - close connections
        InactiveState {
            networker: state.networker,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<ActiveState<T, R>> for ClosedState {
    fn from(mut state: ActiveState<T, R>) -> Self {
        state
            .request_handlers
            .iter_mut()
            .for_each(|(_, ref mut p)| {
                trace!("Terminating ongoing request");
                p.process_event(Some(RequestEvent::Terminate));
            });
        trace!("PoolSM: from active to closed");
        ClosedState {}
    }
}

impl<T: Networker, R: RequestHandler<T>> PoolSM<T, R> {
    pub fn handle_event(self, pe: PoolEvent) -> (Self, Option<PoolUpdate>) {
        let PoolSM { id, config, state } = self;
        let (state, update) = match state {
            PoolState::Inactive(init_state) => match pe {
                PoolEvent::Open(cmd_id, opt_txns) => {
                    match _update_pool_nodes(None, opt_txns, &config) {
                        Ok((merkle_tree, nodes, remotes)) => {
                            init_state.networker.borrow_mut().process_event(Some(
                                NetworkerEvent::NodesStateUpdated(remotes, None),
                            ));
                            (
                                PoolState::Active((init_state, merkle_tree, nodes).into()),
                                Some(PoolUpdate::OpenAck(cmd_id, id, Ok(()))),
                            )
                        }
                        Err(err) => (
                            PoolState::Inactive(init_state),
                            Some(PoolUpdate::OpenAck(cmd_id, id, Err(err))),
                        ),
                    }
                }
                PoolEvent::Refresh(cmd_id, opt_txns) => {
                    match _status_request_handler(
                        init_state.networker.clone(),
                        None,
                        opt_txns,
                        &config,
                    ) {
                        Ok((request_handler, _)) => (
                            PoolState::SyncCatchup((init_state, request_handler, cmd_id).into()),
                            None,
                        ),
                        Err(err) => (
                            PoolState::Inactive(init_state),
                            Some(PoolUpdate::RefreshAck(cmd_id, id, Err(err))),
                        ),
                    }
                }
                PoolEvent::Close(cmd_id) => (
                    PoolState::Closed(init_state.into()),
                    Some(PoolUpdate::CloseAck(cmd_id, id, Ok(()))),
                ),
                _ => (PoolState::Inactive(init_state), None),
            },
            PoolState::SyncCatchup(mut catchup_state) => {
                match pe {
                    PoolEvent::Close(cmd_id) => (
                        PoolState::Closed(catchup_state.into()),
                        Some(PoolUpdate::CloseAck(cmd_id, id, Ok(()))),
                    ),
                    pe => {
                        let oru = catchup_state.request_handler.process_event(
                            RequestEvent::from_pool_event(pe, config.protocol_version),
                        );
                        let cmd_id = catchup_state.cmd_id;
                        match oru {
                            Some(RequestUpdate::CatchupTargetNotFound(err)) => (
                                PoolState::Inactive(catchup_state.into()),
                                Some(PoolUpdate::RefreshAck(cmd_id, id, Err(err))),
                            ),
                            Some(RequestUpdate::CatchupTargetFound(
                                target_mt_root,
                                target_mt_size,
                                merkle_tree,
                            )) => {
                                match _catchup_request_handler(
                                    catchup_state.networker.clone(),
                                    merkle_tree,
                                    target_mt_size,
                                    target_mt_root,
                                    &config,
                                ) {
                                    Ok((request_handler, _)) => {
                                        catchup_state.request_handler = request_handler;
                                        (PoolState::SyncCatchup(catchup_state), None)
                                    }
                                    Err(err) => (
                                        PoolState::Inactive(catchup_state.into()),
                                        Some(PoolUpdate::RefreshAck(cmd_id, id, Err(err))),
                                    ),
                                }
                            }
                            Some(RequestUpdate::Synced(merkle_tree)) => {
                                match _get_nodes_and_remotes(&merkle_tree, &config) {
                                    Ok((nodes, remotes)) => {
                                        catchup_state.networker.borrow_mut().process_event(Some(
                                            NetworkerEvent::NodesStateUpdated(remotes, None),
                                        ));
                                        (
                                            PoolState::Active(
                                                (catchup_state, merkle_tree, nodes).into(),
                                            ),
                                            Some(PoolUpdate::RefreshAck(cmd_id, id, Ok(()))),
                                        )
                                    }
                                    Err(err) => (
                                        PoolState::Inactive(catchup_state.into()),
                                        Some(PoolUpdate::RefreshAck(cmd_id, id, Err(err))),
                                    ),
                                }
                            }
                            _ => (PoolState::SyncCatchup(catchup_state), None),
                        }
                    } /*
                        Some(RequestUpdate::NodesBlacklisted) => (
                            PoolState::Terminated(syncc_state.into()),
                            Some(_open_refresh_ack(
                                cmd_id,
                                id,
                                refresh,
                                Err(err_msg(LedgerErrorKind::InvalidState, "Blacklisted")),
                            )),
                        ),
                      */
                }
            }
            /*PoolState::Terminated(term_state) => match pe {
                PoolEvent::Close(cmd_id) => (
                    PoolState::Closed(term_state.into()),
                    Some(PoolUpdate::CloseAck(cmd_id, Ok(()))),
                ),
                PoolEvent::Refresh(cmd_id) => {
                    match _init_request_handler(term_state.networker.clone(), transactions, &config)
                    {
                        Ok(request_handler, transactions) => (
                            PoolState::GettingCatchupTarget(
                                (term_state, request_handler, cmd_id).into(),
                            ),
                            None,
                        ),
                        Err(err) => (
                            PoolState::Terminated(term_state),
                            Some(_open_refresh_ack(cmd_id, id, true, Err(err))),
                        ),
                    }
                }
                PoolEvent::Timeout(req_id, node_alias) => {
                    if "".eq(&req_id) {
                        term_state
                            .networker
                            .borrow_mut()
                            .process_event(Some(NetworkerEvent::Timeout));
                    } else {
                        warn!(
                            "Unexpected timeout: req_id {}, node_alias {}",
                            req_id, node_alias
                        )
                    }
                    (PoolState::Terminated(term_state), None)
                }
                _ => (PoolState::Terminated(term_state), None),
            },*/
            PoolState::Closed(close_state) => (PoolState::Closed(close_state), None),
            PoolState::Active(mut active_state) => {
                match pe {
                    PoolEvent::PoolOutdated => (
                        PoolState::Inactive(active_state.into()),
                        // FIXME - need a new pool update event?
                        None,
                    ),
                    PoolEvent::Close(cmd_id) => (
                        PoolState::Closed(active_state.into()),
                        Some(PoolUpdate::CloseAck(cmd_id, id, Ok(()))),
                    ),
                    PoolEvent::Refresh(cmd_id, _opt_txns) => {
                        /*let merkle_tree = active_state.merkle_tree.clone();
                        match _status_request_handler(
                            active_state.networker.clone(),
                            Some(active_state.merkle_tree),
                            opt_txns,
                            &config,
                        ) {
                            Ok((request_handler, _)) => {
                                /* FIXME - stay in active state
                                PoolState::SyncCatchup(
                                    (active_state, request_handler, cmd_id).into(),
                                ),
                                None,
                                */
                                /*active_state
                                .request_handlers
                                .insert(req_id.to_string(), request_handler);
                                */
                                active_state.sync_request = Some((cmd_id, request_handler));
                                (PoolState::Active(active_state), None)
                            }
                            Err(err) => (
                                PoolState::Active(active_state),
                                Some(PoolUpdate::RefreshAck(cmd_id, id, Err(err))),
                            ),
                        }*/
                        (
                            PoolState::Active(active_state),
                            Some(PoolUpdate::RefreshAck(cmd_id, id, Ok(()))),
                        )
                    }
                    PoolEvent::SendRequest(cmd_id, _, _, _) => {
                        trace!("received request to send");
                        let re: Option<RequestEvent> =
                            RequestEvent::from_pool_event(pe, config.protocol_version);
                        match re.as_ref().map(|r| r.get_req_id()) {
                            Some(req_id) => {
                                let mut request_handler = R::new(
                                    active_state.networker.clone(),
                                    _get_f(active_state.nodes.len()),
                                    &[cmd_id],
                                    &active_state.nodes,
                                    &config,
                                );
                                request_handler.process_event(re); // FIXME check result
                                active_state
                                    .request_handlers
                                    .insert(req_id.to_string(), request_handler);
                                // FIXME check already exists
                                (PoolState::Active(active_state), None)
                            }
                            None => {
                                let res = Err(err_msg(
                                    LedgerErrorKind::InvalidStructure,
                                    "Request ID not found",
                                ));
                                (
                                    PoolState::Active(active_state),
                                    Some(PoolUpdate::SubmitAck(cmd_id, id, res)),
                                )
                            }
                        }
                    }
                    PoolEvent::NodeReply(ref reply, ref node) => {
                        trace!("received reply from node {:?}: {:?}", node, reply);
                        let re: Option<RequestEvent> =
                            RequestEvent::from_pool_event(pe, config.protocol_version);
                        match re.as_ref().map(|r| r.get_req_id()) {
                            Some(req_id) => {
                                let remove = if let Some(rh) =
                                    active_state.request_handlers.get_mut(&req_id)
                                {
                                    rh.process_event(re);
                                    rh.is_terminal()
                                } else {
                                    false
                                };
                                if remove {
                                    active_state.request_handlers.remove(&req_id);
                                }
                            }
                            None => warn!("Request id not found in Reply"), // : {:?}", reply),
                        };
                        (PoolState::Active(active_state), None)
                    }
                    PoolEvent::Timeout(ref req_id, ref node_alias) => {
                        if let Some(rh) = active_state.request_handlers.get_mut(req_id) {
                            rh.process_event(RequestEvent::from_pool_event(
                                pe,
                                config.protocol_version,
                            ));
                        } else if req_id.is_empty() {
                            active_state
                                .networker
                                .borrow_mut()
                                .process_event(Some(NetworkerEvent::Timeout));
                        } else {
                            warn!(
                                "Unexpected timeout: req_id {}, node_alias {}",
                                req_id, node_alias
                            )
                        }
                        (PoolState::Active(active_state), None)
                    }
                    _ => (PoolState::Active(active_state), None),
                }
            }
        };
        (PoolSM::step(id, config, state), update)
    }

    pub fn is_terminal(&self) -> bool {
        match self.state {
            PoolState::Inactive(_) | PoolState::Active(_) | PoolState::SyncCatchup(_) => false,
            PoolState::Closed(_) => true,
        }
    }
}

pub struct Pool<T: Networker, R: RequestHandler<T>> {
    _pd: PhantomData<(T, R)>,
    worker: Option<JoinHandle<()>>,
    id: PoolHandle,
    config: PoolConfig,
}

impl<T: Networker, R: RequestHandler<T>> Pool<T, R> {
    pub fn new(id: PoolHandle, config: &PoolConfig) -> Self {
        trace!("Pool::new id {:?}, config {:?}", id, config);
        Pool {
            _pd: PhantomData::<(T, R)>,
            worker: None,
            id,
            config: config.clone(),
        }
    }

    pub fn work(&mut self, cmd_socket: zmq::Socket) {
        let id = self.id;
        let config = self.config.clone();
        self.worker = Some(thread::spawn(move || {
            let mut pool_thread: PoolThread<T, R> = PoolThread::new(id, config, cmd_socket);
            pool_thread.work();
        }));
    }

    pub fn get_id(&self) -> PoolHandle {
        self.id
    }
}

struct PoolThread<T: Networker, R: RequestHandler<T>> {
    pool_sm: Option<PoolSM<T, R>>,
    events: VecDeque<PoolEvent>,
    commander: Commander,
    networker: Rc<RefCell<T>>,
}

impl<T: Networker, R: RequestHandler<T>> PoolThread<T, R> {
    pub fn new(id: PoolHandle, config: PoolConfig, cmd_socket: zmq::Socket) -> Self {
        let networker = Rc::new(RefCell::new(T::new(
            config.conn_active_timeout,
            config.conn_request_limit,
        )));
        PoolThread {
            pool_sm: Some(PoolSM::new(id, config, networker.clone())),
            events: VecDeque::new(),
            commander: Commander::new(cmd_socket),
            networker,
        }
    }

    pub fn work(&mut self) {
        loop {
            self._poll();

            if self._loop() {
                break;
            }
        }
    }

    fn _loop(&mut self) -> bool {
        while !self.events.is_empty() {
            let sm = self.pool_sm.take();
            let pe = self.events.pop_front();
            trace!("received pool event: {:?}", pe);
            match (sm, pe) {
                (Some(sm), Some(pe)) => {
                    let (state, _update) = sm.handle_event(pe);
                    // FIXME - do something with update
                    self.pool_sm.replace(state);
                }
                _ => (),
            }
        }
        self.pool_sm
            .as_ref()
            .map(|w| w.is_terminal())
            .unwrap_or(true)
    }

    fn _poll(&mut self) {
        let events = {
            let networker = self.networker.borrow();

            let mut poll_items = networker.get_poll_items();
            //            trace!("prevents: {:?}", poll_items.iter().map(|pi| pi.revents));
            poll_items.push(self.commander.get_poll_item());

            let ((req_id, alias), timeout) = networker.get_timeout();
            //            trace!("next timeout: {:?}", timeout);

            let poll_res = zmq::poll(&mut poll_items, ::std::cmp::max(timeout, 0))
                .map_err(map_err_err!())
                .map_err(|_| unimplemented!() /* FIXME */)
                .unwrap();
            //            trace!("poll_res: {:?}", poll_res);
            if poll_res == 0 {
                self.events.push_back(PoolEvent::Timeout(req_id, alias)); // TODO check duplicate ?
            }
            //            trace!("poll_items: {:?}", poll_items.len());

            let mut events = networker.fetch_events(poll_items.as_slice());
            //            trace!("events: {:?}", events);
            if poll_items[poll_items.len() - 1].is_readable() {
                //TODO move into fetch events?
                events.extend(self.commander.fetch_events());
            }

            events
        };

        self.events.extend(events);
    }
}
*/

/*
fn _get_f(cnt: usize) -> usize {
    if cnt < 4 {
        return 0;
    }
    (cnt - 1) / 3
}
*/

/*
fn _get_merkle_tree(
    merkle_tree: Option<MerkleTree>,
    new_txns: Option<JsonTransactions>,
) -> LedgerResult<MerkleTree> {
    match (merkle_tree, new_txns) {
        (Some(merkle_tree), None) => Ok(merkle_tree),
        (_, Some(new_txns)) => merkle_tree_factory::make_tree(new_txns),
        _ => Err(err_msg(
            LedgerErrorKind::InvalidState,
            "No genesis transactions",
        )),
    }
}

fn _update_pool_nodes(
    merkle_tree: Option<MerkleTree>,
    new_txns: Option<JsonTransactions>,
    config: &PoolConfig,
) -> LedgerResult<(MerkleTree, Nodes, Vec<RemoteNode>)> {
    let merkle_tree = _get_merkle_tree(merkle_tree, new_txns)?;
    let (nodes, remotes) = _get_nodes_and_remotes(&merkle_tree, config)?;
    Ok((merkle_tree, nodes, remotes))
}
*/

/*
fn _status_request_handler<T: Networker, R: RequestHandler<T>>(
    networker: Rc<RefCell<T>>,
    merkle_tree: Option<MerkleTree>,
    new_txns: Option<JsonTransactions>,
    config: &PoolConfig,
) -> LedgerResult<(R, Option<RequestUpdate>)> {
    let merkle_tree = _get_merkle_tree(merkle_tree, new_txns)?;
    let (nodes, remotes) = _get_nodes_and_remotes(&merkle_tree, config)?;
    networker
        .borrow_mut()
        .process_event(Some(NetworkerEvent::NodesStateUpdated(remotes, None)));
    let mut handler = R::new(networker.clone(), _get_f(nodes.len()), &[], &nodes, &config);
    let update = handler.process_event(Some(RequestEvent::StatusReq(merkle_tree)));
    Ok((handler, update))
}

fn _catchup_request_handler<T: Networker, R: RequestHandler<T>>(
    networker: Rc<RefCell<T>>,
    merkle_tree: MerkleTree,
    target_mt_size: usize,
    target_mt_root: Vec<u8>,
    config: &PoolConfig,
) -> LedgerResult<(R, Option<RequestUpdate>)> {
    let (nodes, remotes) = _get_nodes_and_remotes(&merkle_tree, config)?;
    networker
        .borrow_mut()
        .process_event(Some(NetworkerEvent::NodesStateUpdated(remotes, None)));
    let mut handler = R::new(networker.clone(), _get_f(nodes.len()), &[], &nodes, &config);
    let update = handler.process_event(Some(RequestEvent::CatchupReq(
        merkle_tree,
        target_mt_size,
        target_mt_root,
    )));
    Ok((handler, update))
}
*/

/*
fn _get_nodes_and_remotes(
    merkle: &MerkleTree,
    config: &PoolConfig,
) -> LedgerResult<(Nodes, Vec<RemoteNode>)> {
    let nodes = merkle_tree_factory::build_node_state(merkle, config.protocol_version)?;

    Ok(nodes
        .iter()
        .map(|(dest, txn)| {
            let node_alias = txn.txn.data.data.alias.clone();

            let node_verkey = dest
                .as_str()
                .from_base58()
                .map_err(Context::new)
                .to_result(
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
                    let key = blskey
                        .as_str()
                        .from_base58()
                        .map_err(Context::new)
                        .to_result(
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
*/

/*
pub struct ZMQPool {
    pub(super) pool: Pool<ZMQNetworker, RequestHandlerImpl<ZMQNetworker>>,
    pub(super) cmd_socket: zmq::Socket,
}

impl ZMQPool {
    pub fn new(
        pool: Pool<ZMQNetworker, RequestHandlerImpl<ZMQNetworker>>,
        cmd_socket: zmq::Socket,
    ) -> ZMQPool {
        ZMQPool { pool, cmd_socket }
    }
}

impl Drop for ZMQPool {
    fn drop(&mut self) {
        info!("Drop started");

        if let Err(err) = self.cmd_socket.send(COMMAND_EXIT.as_bytes(), zmq::DONTWAIT) {
            warn!(
                "Can't send exit command to pool worker thread (may be already finished) {}",
                err
            );
        }

        // Option worker type and this kludge is workaround for rust
        if let Some(worker) = self.pool.worker.take() {
            info!("Drop wait worker");
            worker.join().unwrap();
        }
        info!("Drop finished");
    }
}
*/

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
