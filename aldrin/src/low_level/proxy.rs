use super::Event;
use super::Reply;
#[cfg(feature = "introspection")]
use crate::core::introspection::Introspection;
use crate::core::{Serialize, ServiceId, ServiceInfo, TypeId};
use crate::error::Error;
use crate::handle::Handle;
use futures_core::stream::{FusedStream, Stream};
#[cfg(feature = "introspection")]
use std::borrow::Cow;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Proxy to a service.
#[derive(Debug)]
pub struct Proxy {
    client: Handle,
    id: ServiceId,
    info: ServiceInfo,
}

impl Proxy {
    /// Creates a new proxy to a service.
    pub async fn new(client: Handle, id: ServiceId) -> Result<Self, Error> {
        let info = client.query_service_info(id).await?;
        Ok(Self { client, id, info })
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

    /// Returns the type id of the proxy's service, if it is known.
    pub fn type_id(&self) -> Option<TypeId> {
        self.info.type_id
    }

    /// Queries the introspection for the proxy's service.
    #[cfg(feature = "introspection")]
    pub async fn query_introspection(&self) -> Result<Option<Cow<'static, Introspection>>, Error> {
        match self.info.type_id {
            Some(type_id) => self.client.query_introspection(type_id).await,
            None => Ok(None),
        }
    }

    /// Calls a function on the service.
    pub fn call<Args>(&self, function: u32, args: &Args) -> Reply
    where
        Args: Serialize + ?Sized,
    {
        self.client.call(self.id, function, args)
    }

    /// Subscribes to an event.
    pub async fn subscribe(&self, _event: u32) -> Result<(), Error> {
        todo!()
    }

    /// Unsubscribe from an event.
    pub async fn unsubscribe(&self, _event: u32) -> Result<(), Error> {
        todo!()
    }

    /// Polls for the next event.
    ///
    /// This function returns `Poll::Pending` even if no events have been subscribed to. `None` is
    /// only returned if either the service was destroyed or the client has shut down.
    pub fn poll_next_event(&mut self, _cx: &mut Context) -> Poll<Option<Event>> {
        todo!()
    }

    /// Returns the next event.
    ///
    /// This function blocks even if no events have been subscribed to. `None` is only returned if
    /// either the service was destroyed or the client has shut down.
    pub async fn next_event(&mut self) -> Option<Event> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }

    /// Indicates whether no more events can be expected.
    ///
    /// When `events_finished` returns `true`, then [`next_event`](Self::next_event) is guaranteed
    /// to return `None`. This happens only if either the service was destroyed or the client has
    /// shut down.
    pub fn events_finished(&self) -> bool {
        todo!()
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
