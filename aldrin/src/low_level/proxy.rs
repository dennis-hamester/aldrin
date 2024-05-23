use super::Reply;
use super::{Event, EventListener};
use crate::core::{Serialize, ServiceId, ServiceInfo};
use crate::error::Error;
use crate::handle::Handle;
use futures_core::stream::{FusedStream, Stream};
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Proxy to a service.
#[derive(Debug)]
pub struct Proxy {
    client: Handle,
    id: ServiceId,
    info: ServiceInfo,
    events: Option<EventListener>,
}

impl Proxy {
    /// Creates a new proxy to a service.
    pub async fn new(client: Handle, id: ServiceId) -> Result<Self, Error> {
        let info = client.query_service_info(id).await?;

        Ok(Self {
            client,
            id,
            info,
            events: None,
        })
    }

    /// Returns a handle to the proxy's client.
    pub fn client(&self) -> &Handle {
        &self.client
    }

    /// Returns the id of the proxy's service.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the version of the proxy's service.
    pub fn version(&self) -> u32 {
        self.info.version
    }

    /// Calls a function on the service.
    pub fn call<Args>(&self, function: u32, args: &Args) -> Reply
    where
        Args: Serialize + ?Sized,
    {
        self.client.call(self.id, function, args)
    }

    /// Subscribes to an event.
    ///
    /// This function returns `true`, if `event` was not already subscribed to. Otherwise `false` is
    /// returned.
    pub async fn subscribe_event(&mut self, event: u32) -> Result<bool, Error> {
        self.events
            .get_or_insert_with(|| EventListener::new(self.client.clone()))
            .subscribe(self.id, event)
            .await
    }

    /// Unsubscribe from an event.
    ///
    /// This function returns `true`, if `event` was subscribed to before the call to this function
    /// and is now unsubscribed from. Otherwise `false` is returned.
    pub fn unsubscribe_event(&mut self, event: u32) -> Result<bool, Error> {
        match self.events {
            Some(ref mut listener) => listener.unsubscribe(self.id, event),
            None => Ok(false),
        }
    }

    /// Polls for the next event.
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<Event>> {
        match self.events {
            Some(ref mut listener) => listener.poll_next_event(cx),
            None => Poll::Ready(None),
        }
    }

    /// Returns the next event.
    pub async fn next_event(&mut self) -> Option<Event> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }

    /// Indicates whether no more events can be expected.
    ///
    /// When `events_finished` returns `true`, then [`next_event`](Self::next_event) is guaranteed
    /// to return `None`.
    pub fn events_finished(&self) -> bool {
        match self.events {
            Some(ref listener) => listener.is_finished(),
            None => false,
        }
    }
}

impl Stream for Proxy {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Event>> {
        self.poll_next_event(cx)
    }
}

impl FusedStream for Proxy {
    fn is_terminated(&self) -> bool {
        self.events_finished()
    }
}
