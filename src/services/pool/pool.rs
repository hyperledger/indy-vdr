use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use failure::Context;

/* use crate::commands::ledger::LedgerCommand;
use crate::commands::pool::PoolCommand;
use crate::commands::Command;
use crate::commands::CommandExecutor; */
use super::commander::Commander;
use super::events::*;
use super::networker::{Networker, ZMQNetworker};
use super::request_handler::{RequestHandler, RequestHandlerImpl};
use super::types::{CommandHandle, LedgerStatus, Nodes, PoolConfig, PoolHandle, RemoteNode};
use crate::domain::pool::ProtocolVersion;
use crate::utils::base58::ToBase58;
use crate::utils::crypto;
use crate::utils::error::prelude::*;
use crate::utils::merkletree::MerkleTree;

use ursa::bls::VerKey as BlsVerKey;
use zmq;

struct PoolSM<T: Networker, R: RequestHandler<T>> {
    id: PoolHandle,
    config: PoolConfig,
    state: PoolState<T, R>,
}

/// Transitions of pool state
/// Initialization -> GettingCatchupTarget, Active, Terminated, Closed
/// GettingCatchupTarget -> SyncCatchup, Active, Terminated, Closed
/// Active -> GettingCatchupTarget, Terminated, Closed
/// SyncCatchup -> Active, Terminated, Closed
/// Terminated -> GettingCatchupTarget, Closed
/// Closed -> Closed
enum PoolState<T: Networker, R: RequestHandler<T>> {
    Initialization(InitializationState<T>),
    GettingCatchupTarget(GettingCatchupTargetState<T, R>),
    Active(ActiveState<T, R>),
    SyncCatchup(SyncCatchupState<T, R>),
    Terminated(TerminatedState<T>),
    Closed(ClosedState),
}

struct InitializationState<T: Networker> {
    networker: Rc<RefCell<T>>,
}

struct GettingCatchupTargetState<T: Networker, R: RequestHandler<T>> {
    networker: Rc<RefCell<T>>,
    request_handler: R,
    cmd_id: CommandHandle,
    refresh: bool,
}

struct ActiveState<T: Networker, R: RequestHandler<T>> {
    networker: Rc<RefCell<T>>,
    request_handlers: HashMap<String, R>,
    nodes: Nodes,
}

struct SyncCatchupState<T: Networker, R: RequestHandler<T>> {
    networker: Rc<RefCell<T>>,
    request_handler: R,
    cmd_id: CommandHandle,
    refresh: bool,
}

struct TerminatedState<T: Networker> {
    networker: Rc<RefCell<T>>,
}

struct ClosedState {}

impl<T: Networker, R: RequestHandler<T>> PoolSM<T, R> {
    pub fn new(id: PoolHandle, config: &PoolConfig, networker: Rc<RefCell<T>>) -> PoolSM<T, R> {
        PoolSM {
            id,
            config: config.clone(),
            state: PoolState::Initialization(InitializationState { networker }),
        }
    }

    pub fn step(id: PoolHandle, config: PoolConfig, state: PoolState<T, R>) -> Self {
        PoolSM { id, config, state }
    }
}

// transitions from Initialization

impl<T: Networker, R: RequestHandler<T>> From<(R, CommandHandle, InitializationState<T>)>
    for GettingCatchupTargetState<T, R>
{
    fn from(
        (request_handler, cmd_id, state): (R, CommandHandle, InitializationState<T>),
    ) -> GettingCatchupTargetState<T, R> {
        trace!("PoolSM: from init to getting catchup target");
        //TODO: fill it up!
        GettingCatchupTargetState {
            networker: state.networker,
            request_handler,
            cmd_id,
            refresh: false,
        }
    }
}

impl<T: Networker> From<InitializationState<T>> for ClosedState {
    fn from(_state: InitializationState<T>) -> ClosedState {
        trace!("PoolSM: from init to closed");
        ClosedState {}
    }
}

impl<T: Networker> From<InitializationState<T>> for TerminatedState<T> {
    fn from(state: InitializationState<T>) -> TerminatedState<T> {
        trace!("PoolSM: from init to terminated");
        TerminatedState {
            networker: state.networker,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<(InitializationState<T>, Nodes)>
    for ActiveState<T, R>
{
    fn from((state, nodes): (InitializationState<T>, Nodes)) -> ActiveState<T, R> {
        trace!("PoolSM: from init to active");
        ActiveState {
            networker: state.networker,
            request_handlers: HashMap::new(),
            nodes,
        }
    }
}

// transitions from GettingCatchupTarget

impl<T: Networker, R: RequestHandler<T>> From<(R, GettingCatchupTargetState<T, R>)>
    for SyncCatchupState<T, R>
{
    fn from((request_handler, state): (R, GettingCatchupTargetState<T, R>)) -> Self {
        trace!("PoolSM: from getting catchup target to sync catchup");
        SyncCatchupState {
            networker: state.networker,
            request_handler,
            cmd_id: state.cmd_id,
            refresh: state.refresh,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<(GettingCatchupTargetState<T, R>, Nodes)>
    for ActiveState<T, R>
{
    fn from((state, nodes): (GettingCatchupTargetState<T, R>, Nodes)) -> Self {
        ActiveState {
            networker: state.networker,
            request_handlers: HashMap::new(),
            nodes,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<GettingCatchupTargetState<T, R>>
    for TerminatedState<T>
{
    fn from(state: GettingCatchupTargetState<T, R>) -> Self {
        trace!("PoolSM: from getting catchup target to terminated");
        TerminatedState {
            networker: state.networker,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<GettingCatchupTargetState<T, R>> for ClosedState {
    fn from(mut state: GettingCatchupTargetState<T, R>) -> Self {
        trace!("PoolSM: from getting catchup target to closed");
        state
            .request_handler
            .process_event(Some(RequestEvent::Terminate));
        ClosedState {}
    }
}

// transitions from Active

impl<T: Networker, R: RequestHandler<T>> From<(ActiveState<T, R>, R, CommandHandle)>
    for GettingCatchupTargetState<T, R>
{
    fn from((state, request_handler, cmd_id): (ActiveState<T, R>, R, CommandHandle)) -> Self {
        trace!("PoolSM: from active to getting catchup target");
        //TODO: close connections!
        GettingCatchupTargetState {
            networker: state.networker,
            cmd_id,
            request_handler,
            refresh: true,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<ActiveState<T, R>> for TerminatedState<T> {
    fn from(state: ActiveState<T, R>) -> Self {
        trace!("PoolSM: from active to terminated");
        TerminatedState {
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

// transitions from SyncCatchup

impl<T: Networker, R: RequestHandler<T>> From<(SyncCatchupState<T, R>, Nodes)>
    for ActiveState<T, R>
{
    fn from((state, nodes): (SyncCatchupState<T, R>, Nodes)) -> Self {
        trace!("PoolSM: from sync catchup to active");
        ActiveState {
            networker: state.networker,
            request_handlers: HashMap::new(),
            nodes,
        }
    }
}

impl<T: Networker, R: RequestHandler<T>> From<SyncCatchupState<T, R>> for TerminatedState<T> {
    fn from(state: SyncCatchupState<T, R>) -> Self {
        trace!("PoolSM: from sync catchup to terminated");
        TerminatedState {
            networker: state.networker,
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

// transitions from Terminated

impl<T: Networker, R: RequestHandler<T>> From<(TerminatedState<T>, R, CommandHandle)>
    for GettingCatchupTargetState<T, R>
{
    fn from((state, request_handler, cmd_id): (TerminatedState<T>, R, CommandHandle)) -> Self {
        trace!("PoolSM: from terminated to getting catchup target");
        GettingCatchupTargetState {
            networker: state.networker,
            cmd_id,
            request_handler,
            refresh: true,
        }
    }
}

impl<T: Networker> From<TerminatedState<T>> for ClosedState {
    fn from(_state: TerminatedState<T>) -> Self {
        trace!("PoolSM: from terminated to closed");
        ClosedState {}
    }
}

impl<T: Networker, R: RequestHandler<T>> PoolSM<T, R> {
    pub fn handle_event(self, pe: PoolEvent) -> Self {
        let PoolSM { id, config, state } = self;
        let state =
            match state {
                PoolState::Initialization(init_state) => match pe {
                    PoolEvent::CheckCache(cmd_id) => {
                        //TODO: check cache freshness
                        let fresh = false;
                        if fresh {
                            //  PoolWrapper::Active(pool.into())
                            unimplemented!()
                        } else {
                            match _get_request_handler_with_ledger_status_sent(
                                init_state.networker.clone(),
                                &config,
                            ) {
                                Ok(request_handler) => {
                                    (PoolState::GettingCatchupTarget(
                                        (request_handler, cmd_id, init_state).into(),
                                    ))
                                }
                                Err(err) => {
                                    // FIXME updater.send(UpdateEvent::OpenAck(cmd_id, id, Err(err)));
                                    PoolState::Terminated(init_state.into())
                                }
                            }
                        }
                    }
                    PoolEvent::Close(cmd_id) => {
                        // FIXME updater.send(UpdateEvent::CloseAck(cmd_id, Ok(())));
                        PoolState::Closed(init_state.into())
                    }
                    _ => PoolState::Initialization(init_state),
                },
                PoolState::GettingCatchupTarget(mut catchup_state) => {
                    let ue = catchup_state
                        .request_handler
                        .process_event(RequestEvent::from_pool_event(pe, config.protocol_version));
                    match ue {
                        Some(UpdateEvent::CatchupTargetNotFound(err)) => {
                            /* FIXME updater.send(_open_refresh_ack(
                                catchup_state.cmd_id,
                                id,
                                catchup_state.refresh,
                                Err(err),
                            ));*/
                            PoolState::Terminated(catchup_state.into())
                        }
                        Some(UpdateEvent::CatchupRestart(merkle_tree)) => {
                            if let Ok((nodes, remotes)) = _get_nodes_and_remotes(&merkle_tree) {
                                catchup_state.networker.borrow_mut().process_event(Some(
                                    NetworkerEvent::NodesStateUpdated(remotes),
                                ));
                                catchup_state.request_handler = R::new(
                                    catchup_state.networker.clone(),
                                    _get_f(nodes.len()),
                                    &[],
                                    &nodes,
                                    &config,
                                );
                                let ls = _ledger_status(&merkle_tree, config.protocol_version);
                                catchup_state.request_handler.process_event(Some(
                                    RequestEvent::LedgerStatus(ls, None, Some(merkle_tree)),
                                ));
                                PoolState::GettingCatchupTarget(catchup_state)
                            } else {
                                PoolState::Terminated(catchup_state.into())
                            }
                        }
                        Some(UpdateEvent::CatchupTargetFound(
                            target_mt_root,
                            target_mt_size,
                            merkle_tree,
                        )) => {
                            if let Ok((nodes, remotes)) = _get_nodes_and_remotes(&merkle_tree) {
                                catchup_state.networker.borrow_mut().process_event(Some(
                                    NetworkerEvent::NodesStateUpdated(remotes),
                                ));
                                let mut request_handler = R::new(
                                    catchup_state.networker.clone(),
                                    _get_f(nodes.len()),
                                    &[],
                                    &nodes,
                                    &config,
                                );
                                request_handler.process_event(Some(RequestEvent::CatchupReq(
                                    merkle_tree,
                                    target_mt_size,
                                    target_mt_root,
                                )));
                                PoolState::SyncCatchup((request_handler, catchup_state).into())
                            } else {
                                PoolState::Terminated(catchup_state.into())
                            }
                        }
                        Some(UpdateEvent::Synced(merkle_tree)) => {
                            if let Ok((nodes, remotes)) = _get_nodes_and_remotes(&merkle_tree) {
                                catchup_state.networker.borrow_mut().process_event(Some(
                                    NetworkerEvent::NodesStateUpdated(remotes),
                                ));
                                /* FIXME updater.send(_open_refresh_ack(
                                    catchup_state.cmd_id,
                                    id,
                                    catchup_state.refresh,
                                    Ok(()),
                                ));*/
                                PoolState::Active((catchup_state, nodes).into())
                            } else {
                                PoolState::Terminated(catchup_state.into())
                            }
                        }
                        _ => match pe {
                            PoolEvent::Close(cmd_id) => {
                                // FIXME updater.send(UpdateEvent::CloseAck(cmd_id, Ok(())));
                                PoolState::Closed(catchup_state.into())
                            }
                            _ => PoolState::GettingCatchupTarget(catchup_state),
                        },
                    }
                }
                PoolState::Terminated(term_state) => match pe {
                    PoolEvent::Close(cmd_id) => {
                        // FIXME updater.send(UpdateEvent::CloseAck(cmd_id, Ok(())));
                        PoolState::Closed(term_state.into())
                    }
                    PoolEvent::Refresh(cmd_id) => {
                        if let Ok(request_handler) = _get_request_handler_with_ledger_status_sent(
                            term_state.networker.clone(),
                            &config,
                        ) {
                            PoolState::GettingCatchupTarget(
                                (term_state, request_handler, cmd_id).into(),
                            )
                        } else {
                            PoolState::Terminated(term_state)
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
                        PoolState::Terminated(term_state)
                    }
                    _ => PoolState::Terminated(term_state),
                },
                PoolState::Closed(close_state) => PoolState::Closed(close_state),
                PoolState::Active(mut active_state) => {
                    match pe {
                        PoolEvent::PoolOutdated => PoolState::Terminated(active_state.into()),
                        PoolEvent::Close(cmd_id) => {
                            // FIXME updater.send(UpdateEvent::CloseAck(cmd_id, Ok(())));
                            PoolState::Closed(active_state.into())
                        }
                        PoolEvent::Refresh(cmd_id) => {
                            if let Ok(request_handler) =
                                _get_request_handler_with_ledger_status_sent(
                                    active_state.networker.clone(),
                                    &config,
                                )
                            {
                                PoolState::GettingCatchupTarget(
                                    (active_state, request_handler, cmd_id).into(),
                                )
                            } else {
                                PoolState::Terminated(active_state.into())
                            }
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
                                    request_handler.process_event(re);
                                    active_state
                                        .request_handlers
                                        .insert(req_id.to_string(), request_handler);
                                    // FIXME check already exists
                                }
                                None => {
                                    let res = Err(err_msg(
                                        LedgerErrorKind::InvalidStructure,
                                        "Request id not found",
                                    ));
                                    // FIXME updater.send(UpdateEvent::SubmitAck(cmd_id, res));
                                }
                            };
                            PoolState::Active(active_state)
                        }
                        PoolEvent::NodeReply(reply, node) => {
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
                                None => warn!("Request id not found in Reply: {:?}", reply),
                            };
                            PoolState::Active(active_state)
                        }
                        PoolEvent::Timeout(req_id, node_alias) => {
                            if let Some(rh) = active_state.request_handlers.get_mut(&req_id) {
                                rh.process_event(RequestEvent::from_pool_event(
                                    pe,
                                    config.protocol_version,
                                ));
                            } else if "".eq(&req_id) {
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
                            PoolState::Active(active_state)
                        }
                        _ => PoolState::Active(active_state),
                    }
                }
                PoolState::SyncCatchup(mut syncc_state) => {
                    let ue = syncc_state
                        .request_handler
                        .process_event(RequestEvent::from_pool_event(pe, config.protocol_version));
                    match ue {
                        Some(UpdateEvent::NodesBlacklisted) => {
                            PoolState::Terminated(syncc_state.into())
                        }
                        Some(UpdateEvent::Synced(merkle)) => {
                            if let Ok((nodes, remotes)) =
                                _get_nodes_and_remotes(&merkle).map_err(map_err_err!())
                            {
                                syncc_state.networker.borrow_mut().process_event(Some(
                                    NetworkerEvent::NodesStateUpdated(remotes),
                                ));
                                /* FIXME updater.send(_open_refresh_ack(
                                    syncc_state.cmd_id,
                                    id,
                                    syncc_state.refresh,
                                    Ok(()),
                                ));*/
                                PoolState::Active((syncc_state, nodes).into())
                            } else {
                                PoolState::Terminated(syncc_state.into())
                            }
                        }
                        _ => match pe {
                            PoolEvent::Close(cmd_id) => {
                                // FIXME updater.send(UpdateEvent::CloseAck(cmd_id, Ok(())));
                                PoolState::Closed(syncc_state.into())
                            }
                            _ => PoolState::SyncCatchup(syncc_state),
                        },
                    }
                }
            };
        PoolSM::step(id, config, state)
    }

    pub fn is_terminal(&self) -> bool {
        match self.state {
            PoolState::Initialization(_)
            | PoolState::GettingCatchupTarget(_)
            | PoolState::Active(_)
            | PoolState::SyncCatchup(_)
            | PoolState::Terminated(_) => false,
            PoolState::Closed(_) => true,
        }
    }
}

impl UpdateHandler for Sender<UpdateEvent> {
    fn send(&self, update: UpdateEvent) -> LedgerResult<()> {
        self.send(update).to_result(
            LedgerErrorKind::IOError,
            "Error returning pool update event",
        )
    }
}

pub struct Pool<T: Networker, R: RequestHandler<T>> {
    _pd: PhantomData<(T, R)>,
    worker: Option<JoinHandle<()>>,
    id: PoolHandle,
    config: PoolConfig,
}

impl<T: Networker, R: RequestHandler<T>> Pool<T, R>
where
    T: Sync,
    R: Sync,
{
    pub fn new(id: PoolHandle, config: &PoolConfig) -> Self {
        trace!("Pool::new id {:?}, config {:?}", id, config);
        Pool {
            _pd: PhantomData::<(T, R)>,
            worker: None,
            id,
            config: config.clone(),
        }
    }

    pub fn work(&self, cmd_socket: zmq::Socket) {
        let config = &self.config;
        self.worker = Some(thread::spawn(move || {
            let mut pool_thread: PoolThread<T, R> = PoolThread::new(self.id, config, cmd_socket);
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
    pub fn new(id: PoolHandle, config: &PoolConfig, cmd_socket: zmq::Socket) -> Self {
        let networker = Rc::new(RefCell::new(T::new(
            config.conn_active_timeout,
            config.conn_limit,
            config.preordered_nodes.clone(),
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
            let pe = self.events.pop_front();
            trace!("received pool event: {:?}", pe);
            match pe {
                Some(pe) => {
                    self.pool_sm = self.pool_sm.take().map(|w| w.handle_event(pe));
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

fn _get_f(cnt: usize) -> usize {
    if cnt < 4 {
        return 0;
    }
    (cnt - 1) / 3
}

fn _get_request_handler_with_ledger_status_sent<T: Networker, R: RequestHandler<T>>(
    networker: Rc<RefCell<T>>,
    config: &PoolConfig,
) -> LedgerResult<R> {
    let mut merkle = merkle_tree_factory::create(pool_name)?;

    let (nodes, remotes) = match _get_nodes_and_remotes(&merkle) {
        Ok(n) => n,
        Err(err) => match merkle_tree_factory::drop_cache(pool_name) {
            Ok(_) => {
                merkle = merkle_tree_factory::create(pool_name)?;
                _get_nodes_and_remotes(&merkle)?
            }
            Err(_) => {
                return Err(err);
            }
        },
    };
    networker
        .borrow_mut()
        .process_event(Some(NetworkerEvent::NodesStateUpdated(remotes)));
    let mut request_handler = R::new(networker.clone(), _get_f(nodes.len()), &[], &nodes, &config);
    // FIXME - let RequestEvent produce the ledger status event in response to a PoolEvent?
    let ls = _ledger_status(&merkle, config.protocol_version);
    request_handler.process_event(Some(RequestEvent::LedgerStatus(ls, None, Some(merkle))));
    Ok(request_handler)
}

fn _ledger_status(merkle: &MerkleTree, protocol_version: ProtocolVersion) -> LedgerStatus {
    LedgerStatus {
        txnSeqNo: merkle.count(),
        merkleRoot: merkle.root_hash().as_slice().to_base58(),
        ledgerId: 0,
        ppSeqNo: None,
        viewNo: None,
        protocolVersion: Some(protocol_version as usize),
    }
}

fn _get_nodes_and_remotes(merkle: &MerkleTree) -> LedgerResult<(Nodes, Vec<RemoteNode>)> {
    let nodes = merkle_tree_factory::build_node_state(merkle)?;

    Ok(nodes
        .iter()
        .map(|(_, txn)| {
            let node_alias = txn.txn.data.data.alias.clone();

            let node_verkey = txn
                .txn
                .data
                .dest
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

fn _open_refresh_ack(
    cmd_id: CommandHandle,
    id: PoolHandle,
    is_refresh: bool,
    res: LedgerResult<()>,
) -> UpdateEvent {
    trace!("PoolSM: from getting catchup target to active");
    if is_refresh {
        UpdateEvent::RefreshAck(cmd_id, res)
    } else {
        UpdateEvent::OpenAck(cmd_id, id, res)
    }
}

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

#[cfg(test)]
mod tests {
    use crate::domain::pool::{DEFAULT_PROTOCOL_VERSION, NUMBER_READ_NODES};
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

    const TEST_POOL_CONFIG: PoolConfig = PoolConfig {
        conn_active_timeout: 0,
        conn_limit: NUMBER_READ_NODES,
        freshness_threshold: 0,
        timeout: 0,
        extended_timeout: 0,
        number_read_nodes: NUMBER_READ_NODES,
        protocol_version: DEFAULT_PROTOCOL_VERSION,
        preordered_nodes: vec![],
    };

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
        // use crate::domain::pool::NUMBER_READ_NODES;

        #[test]
        pub fn pool_wrapper_new_initialization_works() {
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
        pub fn pool_wrapper_close_works_from_initialization() {
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
