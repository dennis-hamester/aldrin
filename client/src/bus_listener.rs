#[cfg(test)]
mod test;

use crate::error::Error;
use crate::handle::Handle;
use aldrin_proto::{BusEvent, BusListenerCookie, BusListenerFilter, BusListenerScope};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_core::stream::{FusedStream, Stream};
use std::collections::HashSet;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// TODO
#[derive(Debug)]
pub struct BusListener {
    cookie: BusListenerCookie,
    client: Handle,
    filters: HashSet<BusListenerFilter>,
    scope: Option<BusListenerScope>,
    pending_started: usize,
    pending_stopped: usize,
    pending_current_finished: usize,
    events: UnboundedReceiver<BusListenerEvent>,
}

impl BusListener {
    pub(crate) fn new(
        cookie: BusListenerCookie,
        client: Handle,
        events: UnboundedReceiver<BusListenerEvent>,
    ) -> Self {
        Self {
            cookie,
            client,
            filters: HashSet::new(),
            scope: None,
            pending_started: 0,
            pending_stopped: 0,
            pending_current_finished: 0,
            events,
        }
    }

    /// TODO
    pub fn handle(&self) -> &Handle {
        &self.client
    }

    /// TODO
    pub async fn destroy(&mut self) -> Result<(), Error> {
        self.client.destroy_bus_listener(self.cookie).await?;

        self.events.close();
        self.pending_started = 0;
        self.pending_stopped = 0;
        self.pending_current_finished = 0;

        Ok(())
    }

    /// TODO
    pub fn add_filter(&mut self, filter: BusListenerFilter) -> Result<(), Error> {
        if self.filters.insert(filter) {
            self.client.add_bus_listener_filter(self.cookie, filter)
        } else {
            Ok(())
        }
    }

    /// TODO
    pub fn remove_filter(&mut self, filter: BusListenerFilter) -> Result<(), Error> {
        if self.filters.remove(&filter) {
            self.client.remove_bus_listener_filter(self.cookie, filter)
        } else {
            Ok(())
        }
    }

    /// TODO
    pub fn clear_filters(&mut self) -> Result<(), Error> {
        if !self.filters.is_empty() {
            self.filters.clear();
            self.client.clear_bus_listener_filters(self.cookie)
        } else {
            Ok(())
        }
    }

    /// TODO
    pub fn filters(&self) -> impl Iterator<Item = BusListenerFilter> + '_ {
        self.filters.iter().copied()
    }

    /// TODO
    pub fn has_filter(&self, filter: BusListenerFilter) -> bool {
        self.filters.contains(&filter)
    }

    /// TODO
    pub fn has_any_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// TODO
    pub fn has_no_filters(&self) -> bool {
        self.filters.is_empty()
    }

    /// TODO
    pub fn num_filters(&self) -> usize {
        self.filters.len()
    }

    /// TODO
    pub async fn start(&mut self, scope: BusListenerScope) -> Result<(), Error> {
        self.client.start_bus_listener(self.cookie, scope).await?;
        self.pending_started += 1;

        if scope.includes_current() {
            self.pending_current_finished += 1;
        }

        Ok(())
    }

    /// TODO
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.client.stop_bus_listener(self.cookie).await?;
        self.pending_stopped += 1;
        Ok(())
    }

    /// TODO
    pub fn scope(&self) -> Option<BusListenerScope> {
        self.scope
    }

    /// TODO
    pub fn finished(&self) -> bool {
        self.events.is_terminated()
            || (!self.includes_new()
                && (self.pending_started == 0)
                && (self.pending_stopped == 0)
                && (self.pending_current_finished == 0))
    }

    /// TODO
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<BusEvent>> {
        loop {
            if self.finished() {
                break Poll::Ready(None);
            }

            match Pin::new(&mut self.events).poll_next(cx) {
                Poll::Ready(Some(BusListenerEvent::Started(scope))) => {
                    self.scope = Some(scope);
                    self.pending_started -= 1;
                }

                Poll::Ready(Some(BusListenerEvent::Stopped)) => {
                    self.scope = None;
                    self.pending_stopped -= 1;
                }

                Poll::Ready(Some(BusListenerEvent::Event(event))) => {
                    return Poll::Ready(Some(event))
                }

                Poll::Ready(Some(BusListenerEvent::CurrentFinished)) => {
                    self.pending_current_finished -= 1;
                }

                Poll::Ready(None) => break Poll::Ready(None),
                Poll::Pending => break Poll::Pending,
            }
        }
    }

    /// TODO
    pub async fn next_event(&mut self) -> Option<BusEvent> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }

    fn includes_new(&self) -> bool {
        self.scope
            .map(BusListenerScope::includes_new)
            .unwrap_or(false)
    }
}

impl Drop for BusListener {
    fn drop(&mut self) {
        self.client.destroy_bus_listener_now(self.cookie);
    }
}

impl Stream for BusListener {
    type Item = BusEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<BusEvent>> {
        self.poll_next_event(cx)
    }
}

impl FusedStream for BusListener {
    fn is_terminated(&self) -> bool {
        self.finished()
    }
}

#[derive(Debug)]
pub(crate) struct BusListenerHandle {
    filters: HashSet<BusListenerFilter>,
    events: UnboundedSender<BusListenerEvent>,
    scope: Option<BusListenerScope>,
    current_finished: bool,
}

impl BusListenerHandle {
    pub fn new(events: UnboundedSender<BusListenerEvent>) -> Self {
        Self {
            filters: HashSet::new(),
            events,
            scope: None,
            current_finished: false,
        }
    }

    pub fn add_filter(&mut self, filter: BusListenerFilter) {
        self.filters.insert(filter);
    }

    pub fn remove_filter(&mut self, filter: BusListenerFilter) {
        self.filters.remove(&filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub fn start(&mut self, scope: BusListenerScope) -> bool {
        if self.scope.is_none() {
            self.scope = Some(scope);
            self.current_finished = !scope.includes_current();
            self.events
                .unbounded_send(BusListenerEvent::Started(scope))
                .ok();
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) -> bool {
        if self.scope.is_some() {
            self.scope = None;
            self.events.unbounded_send(BusListenerEvent::Stopped).ok();
            true
        } else {
            false
        }
    }

    pub fn current_finished(&mut self) -> bool {
        if !self.current_finished {
            self.events
                .unbounded_send(BusListenerEvent::CurrentFinished)
                .ok();
            self.current_finished = true;
            true
        } else {
            false
        }
    }

    pub fn emit_current(&self, event: BusEvent) -> bool {
        if self.includes_current() && !self.current_finished {
            self.events
                .unbounded_send(BusListenerEvent::Event(event))
                .ok();
            true
        } else {
            false
        }
    }

    pub fn emit_new_if_matches(&self, event: BusEvent) {
        if self.includes_new() && self.matches_filters(event) {
            self.events
                .unbounded_send(BusListenerEvent::Event(event))
                .ok();
        }
    }

    fn includes_current(&self) -> bool {
        self.scope
            .map(BusListenerScope::includes_current)
            .unwrap_or(false)
    }

    fn includes_new(&self) -> bool {
        self.scope
            .map(BusListenerScope::includes_new)
            .unwrap_or(false)
    }

    fn matches_filters(&self, event: BusEvent) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.matches_event(event))
    }
}

#[derive(Debug)]
pub(crate) enum BusListenerEvent {
    Started(BusListenerScope),
    Stopped,
    Event(BusEvent),
    CurrentFinished,
}
