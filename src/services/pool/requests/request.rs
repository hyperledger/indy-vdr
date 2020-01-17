use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures::channel::mpsc::Receiver;
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};

use pin_utils::unsafe_pinned;

use crate::utils::error::prelude::*;

use super::networker::{Networker, NetworkerEvent};
use super::types::Nodes;
use super::{
    RequestEvent, RequestExtEvent, RequestState, RequestTimeout, RequestTiming, TimingResult,
};

new_handle_type!(RequestHandle, RQ_COUNTER);

#[must_use = "requests do nothing unless polled"]
pub trait PoolRequest: std::fmt::Debug + Stream<Item = RequestEvent> + FusedStream + Unpin {
    fn clean_timeout(&self, node_alias: String) -> LedgerResult<()>;
    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()>;
    fn get_timing(&self) -> Option<TimingResult>;
    fn is_active(&self) -> bool;
    fn node_aliases(&self) -> Vec<String>;
    fn node_count(&self) -> usize;
    fn node_keys(&self) -> LedgerResult<Nodes>;
    fn send_to_all(&mut self, timeout: RequestTimeout) -> LedgerResult<()>;
    fn send_to_any(&mut self, count: usize, timeout: RequestTimeout) -> LedgerResult<Vec<String>>;
    fn send_to(
        &mut self,
        node_aliases: Vec<String>,
        timeout: RequestTimeout,
    ) -> LedgerResult<Vec<String>>;
}

pub struct PoolRequestImpl {
    handle: RequestHandle,
    events: Option<Receiver<RequestExtEvent>>,
    networker: Arc<RwLock<dyn Networker>>,
    node_aliases: Vec<String>,
    send_count: usize,
    state: RequestState,
    timing: RequestTiming,
}

impl PoolRequestImpl {
    unsafe_pinned!(events: Option<Receiver<RequestExtEvent>>);

    pub fn new(
        handle: RequestHandle,
        events: Receiver<RequestExtEvent>,
        networker: Arc<RwLock<dyn Networker>>,
        node_aliases: Vec<String>,
    ) -> Self {
        Self {
            handle,
            events: Some(events),
            networker,
            node_aliases,
            send_count: 0,
            state: RequestState::NotStarted,
            timing: RequestTiming::new(),
        }
    }

    fn trigger(&self, event: NetworkerEvent) -> LedgerResult<()> {
        self.networker
            .read()
            .map_err(|_| err_msg(LedgerErrorKind::InvalidState, "Error sending to networker"))?
            .send(event)
    }
}

impl PoolRequest for PoolRequestImpl {
    fn clean_timeout(&self, node_alias: String) -> LedgerResult<()> {
        self.trigger(NetworkerEvent::CleanTimeout(self.handle, node_alias))
    }

    fn extend_timeout(&self, node_alias: String, timeout: RequestTimeout) -> LedgerResult<()> {
        self.trigger(NetworkerEvent::ExtendTimeout(
            self.handle,
            node_alias,
            timeout,
        ))
    }

    fn get_timing(&self) -> Option<TimingResult> {
        self.timing.get_result()
    }

    fn is_active(&self) -> bool {
        self.state == RequestState::Active
    }

    fn node_aliases(&self) -> Vec<String> {
        self.node_aliases.clone()
    }

    fn node_count(&self) -> usize {
        self.node_aliases.len()
    }

    fn node_keys(&self) -> LedgerResult<Nodes> {
        // FIXME - remove nodes that aren't present in node_aliases?
        Ok(self
            .networker
            .read()
            .map_err(|_| err_msg(LedgerErrorKind::InvalidState, "Error fetching node keys"))?
            .node_keys())
    }

    fn send_to_all(&mut self, timeout: RequestTimeout) -> LedgerResult<()> {
        let aliases = self.node_aliases();
        let count = aliases.len();
        self.trigger(NetworkerEvent::Dispatch(self.handle, aliases, timeout))?;
        self.send_count += count;
        Ok(())
    }

    fn send_to_any(&mut self, count: usize, timeout: RequestTimeout) -> LedgerResult<Vec<String>> {
        let aliases = self.node_aliases();
        let max = std::cmp::min(self.send_count + count, aliases.len());
        let min = std::cmp::min(self.send_count, max);
        trace!("send to any {} {} {:?}", min, max, aliases);
        let nodes = (min..max)
            .map(|idx| aliases[idx].clone())
            .collect::<Vec<String>>();
        if nodes.len() > 0 {
            self.trigger(NetworkerEvent::Dispatch(
                self.handle,
                nodes.clone(),
                timeout,
            ))?;
            self.send_count += nodes.len();
        }
        Ok(nodes)
    }

    fn send_to(
        &mut self,
        node_aliases: Vec<String>,
        timeout: RequestTimeout,
    ) -> LedgerResult<Vec<String>> {
        let aliases = self
            .node_aliases()
            .iter()
            .filter(|n| node_aliases.contains(n))
            .cloned()
            .collect::<Vec<String>>();
        if aliases.len() > 0 {
            self.trigger(NetworkerEvent::Dispatch(
                self.handle,
                aliases.clone(),
                timeout,
            ))?;
            self.send_count += aliases.len();
        }
        Ok(aliases)
    }
}

impl std::fmt::Debug for PoolRequestImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PoolRequest({}, state={})",
            self.handle.value(),
            self.state
        )
    }
}

impl Drop for PoolRequestImpl {
    fn drop(&mut self) {
        trace!("Finish dropped request: {}", self.handle);
        self.trigger(NetworkerEvent::FinishRequest(self.handle))
            .unwrap_or(()) // don't mind if the receiver disconnected
    }
}

impl Stream for PoolRequestImpl {
    type Item = RequestEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            trace!("poll_next");
            match self.state {
                RequestState::NotStarted => {
                    if let Some(events) = self.as_mut().events().as_pin_mut() {
                        match events.poll_next(cx) {
                            Poll::Ready(val) => {
                                if let Some(RequestExtEvent::Init()) = val {
                                    trace!("Request active {}", self.handle);
                                    self.state = RequestState::Active
                                } else {
                                    trace!("Request aborted {}", self.handle);
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
                                Some(RequestExtEvent::Sent(alias, when)) => {
                                    trace!("{} was sent to {}", self.handle, alias);
                                    self.timing.sent(&alias, when)
                                }
                                Some(RequestExtEvent::Received(alias, message, meta, when)) => {
                                    trace!("{} response from {}", self.handle, alias);
                                    self.timing.received(&alias, when);
                                    return Poll::Ready(Some(RequestEvent::Received(
                                        alias, message, meta,
                                    )));
                                }
                                Some(RequestExtEvent::Timeout(alias)) => {
                                    trace!("{} timed out {}", self.handle, alias);
                                    return Poll::Ready(Some(RequestEvent::Timeout(alias)));
                                }
                                _ => {
                                    trace!("{} terminated", self.handle);
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

impl FusedStream for PoolRequestImpl {
    fn is_terminated(&self) -> bool {
        self.state == RequestState::Terminated
    }
}
