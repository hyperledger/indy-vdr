use std::collections::HashMap;
use std::iter::FromIterator;
use std::pin::Pin;

use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::{FusedStream, Stream};
use futures_util::task::{Context, Poll};

use pin_utils::unsafe_pinned;

use crate::common::error::prelude::*;
use crate::config::PoolConfig;

use super::networker::{Networker, NetworkerEvent};
use super::types::{RequestHandle, TimingResult, VerifierKeys};
use super::PoolSetup;
use super::{RequestEvent, RequestExtEvent, RequestState, RequestTiming};

/// Base trait for pool request implementations
#[must_use = "requests do nothing unless polled"]
pub trait PoolRequest: std::fmt::Debug + Stream<Item = RequestEvent> + FusedStream + Unpin {
    fn clean_timeout(&self, node_alias: String) -> VdrResult<()>;
    fn extend_timeout(&self, node_alias: String, timeout: i64) -> VdrResult<()>;
    fn get_timing(&self) -> Option<TimingResult>;
    fn is_active(&self) -> bool;
    fn node_count(&self) -> usize;
    fn node_keys(&self) -> VerifierKeys;
    fn node_order(&self) -> Vec<String>;
    fn pool_config(&self) -> PoolConfig;
    fn send_to_all(&mut self, timeout: i64) -> VdrResult<()>;
    fn send_to_any(&mut self, count: usize, timeout: i64) -> VdrResult<Vec<String>>;
    fn send_to(&mut self, node_aliases: Vec<String>, timeout: i64) -> VdrResult<Vec<String>>;
}

/// Default `PoolRequestImpl` used by `PoolImpl`
pub struct PoolRequestImpl<S: AsRef<PoolSetup>, T: Networker> {
    handle: RequestHandle,
    events: Option<UnboundedReceiver<RequestExtEvent>>,
    node_order: Vec<String>,
    pool_setup: S,
    networker: T,
    send_count: usize,
    state: RequestState,
    timing: RequestTiming,
}

impl<S, T> PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    unsafe_pinned!(events: Option<UnboundedReceiver<RequestExtEvent>>);

    pub(crate) fn new(
        handle: RequestHandle,
        events: UnboundedReceiver<RequestExtEvent>,
        pool_setup: S,
        networker: T,
        node_order: Vec<String>,
    ) -> Self {
        Self {
            handle,
            events: Some(events),
            pool_setup,
            networker,
            node_order,
            send_count: 0,
            state: RequestState::NotStarted,
            timing: RequestTiming::new(),
        }
    }

    fn trigger(&self, event: NetworkerEvent) -> VdrResult<()> {
        self.networker.send(event)
    }
}

impl<S, T> Unpin for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
}

impl<S, T> PoolRequest for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    fn clean_timeout(&self, node_alias: String) -> VdrResult<()> {
        self.trigger(NetworkerEvent::CleanTimeout(self.handle, node_alias))
    }

    fn extend_timeout(&self, node_alias: String, timeout: i64) -> VdrResult<()> {
        self.trigger(NetworkerEvent::ExtendTimeout(
            self.handle,
            node_alias,
            timeout,
        ))
    }

    fn get_timing(&self) -> Option<TimingResult> {
        self.timing.result()
    }

    fn is_active(&self) -> bool {
        self.state == RequestState::Active
    }

    fn node_order(&self) -> Vec<String> {
        self.node_order.clone()
    }

    fn node_count(&self) -> usize {
        self.node_order.len()
    }

    fn node_keys(&self) -> VerifierKeys {
        let verifiers = &self.pool_setup.as_ref().verifiers;
        HashMap::from_iter(self.node_order.iter().flat_map(|alias| {
            verifiers.get(alias).and_then(|entry| {
                entry
                    .bls_key
                    .as_ref()
                    .map(|bls_key| (alias.clone(), bls_key.clone()))
            })
        }))
    }

    fn pool_config(&self) -> PoolConfig {
        self.pool_setup.as_ref().config.clone()
    }

    fn send_to_all(&mut self, timeout: i64) -> VdrResult<()> {
        let aliases = self.node_order();
        let count = aliases.len();
        trace!("Send to all {} {:?}", self.handle, aliases);
        self.trigger(NetworkerEvent::Dispatch(self.handle, aliases, timeout))?;
        self.send_count += count;
        Ok(())
    }

    fn send_to_any(&mut self, count: usize, timeout: i64) -> VdrResult<Vec<String>> {
        let aliases = self.node_order();
        let max = std::cmp::min(self.send_count + count, aliases.len());
        let min = std::cmp::min(self.send_count, max);
        trace!(
            "Send to any {} {}-{} of {:?}",
            self.handle,
            min,
            max,
            aliases
        );
        let nodes = (min..max)
            .map(|idx| aliases[idx].clone())
            .collect::<Vec<String>>();
        if !nodes.is_empty() {
            self.trigger(NetworkerEvent::Dispatch(
                self.handle,
                nodes.clone(),
                timeout,
            ))?;
            self.send_count += nodes.len();
        }
        Ok(nodes)
    }

    fn send_to(&mut self, node_aliases: Vec<String>, timeout: i64) -> VdrResult<Vec<String>> {
        let aliases = self
            .node_order
            .iter()
            .filter(|n| node_aliases.contains(n))
            .cloned()
            .collect::<Vec<String>>();
        if !aliases.is_empty() {
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

impl<S, T> std::fmt::Debug for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PoolRequest({}, state={})", *self.handle, self.state)
    }
}

impl<S, T> Drop for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    fn drop(&mut self) {
        trace!("Finish dropped request: {}", self.handle);
        self.trigger(NetworkerEvent::FinishRequest(self.handle))
            .unwrap_or(()) // don't mind if the receiver disconnected
    }
}

impl<S, T> Stream for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    type Item = RequestEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            trace!("PoolRequestImpl::poll_next");
            match self.state {
                RequestState::NotStarted => {
                    if let Some(events) = self.as_mut().events().as_pin_mut() {
                        match events.poll_next(cx) {
                            Poll::Ready(val) => {
                                if let Some(RequestExtEvent::Init) = val {
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

impl<S, T> FusedStream for PoolRequestImpl<S, T>
where
    S: AsRef<PoolSetup>,
    T: Networker,
{
    fn is_terminated(&self) -> bool {
        self.state == RequestState::Terminated
    }
}
