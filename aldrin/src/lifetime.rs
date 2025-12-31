#[cfg(test)]
mod test;

use crate::bus_listener::BusListenerEvent;
use crate::{Error, Handle, Object};
#[cfg(feature = "introspection")]
use aldrin_core::introspection::{Introspectable, LexicalId, References, ir};
use aldrin_core::tags::{self, PrimaryTag};
use aldrin_core::{
    BusEvent, BusListenerCookie, BusListenerFilter, BusListenerScope, Deserialize,
    DeserializeError, Deserializer, ObjectId, ObjectUuid, Serialize, SerializeError, Serializer,
};
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::future::FusedFuture;
use futures_core::stream::Stream;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A scope that notifies other clients when ends.
///
/// Aldrin services are inherently stateless with respect to the clients that are using
/// them. Sometimes it becomes necessary to know when a client is no longer interested in using a
/// service.
///
/// Scopes and [`Lifetime`s](Lifetime) solve this problem in a robust way. One client can create a
/// `LifetimeScope` while another binds a `Lifetime` to it. The `Lifetime` will then notify when the
/// scope ends. This can be triggered explicitly ([`end`](Self::end)) or implicitly by dropping the
/// scope. A scope also ends when the client that owns it disconnects from be bus for some reason.
///
/// # Examples
///
/// ```
/// # use aldrin_test::tokio::TestBroker;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut broker = TestBroker::new();
/// # let client1 = broker.add_client().await;
/// # let client2 = broker.add_client().await;
/// // Assume client 1 is using a service from client 2. In turn, client 2 needs to know when
/// // client 1 is done.
///
/// // Client 1 creates a scope and passes its id to client 2.
/// let scope = client1.create_lifetime_scope().await?;
/// let id = scope.id();
///
/// // Client 2 creates a lifetime from the id.
/// let lifetime = client2.create_lifetime(id).await?;
///
/// tokio::spawn(async move {
///     // Move in the scope and do some work. The scope is dropped at the end.
///     let _scope = scope;
/// });
///
/// // Resolves when the associated scope ends.
/// lifetime.await;
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct LifetimeScope {
    object: Object,
}

impl LifetimeScope {
    /// Creates a new scope.
    pub async fn new(client: &Handle) -> Result<Self, Error> {
        client.create_lifetime_scope().await
    }

    pub(crate) fn new_impl(object: Object) -> Self {
        Self { object }
    }

    /// Return the scope's id.
    pub fn id(&self) -> LifetimeId {
        LifetimeId(self.object.id())
    }

    /// Returns a handle to the client that was used to create the scope.
    pub fn client(&self) -> &Handle {
        self.object.client()
    }

    /// Ends the scope.
    ///
    /// If the scope has already ended, [`Error::InvalidLifetime`] is returned.
    pub async fn end(&self) -> Result<(), Error> {
        self.object.destroy().await.map_err(|e| {
            if e == Error::InvalidObject {
                Error::InvalidLifetime
            } else {
                e
            }
        })
    }
}

impl PrimaryTag for LifetimeScope {
    type Tag = tags::ObjectId;
}

impl Serialize<tags::ObjectId> for LifetimeScope {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<tags::ObjectId> for &LifetimeScope {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(self.id())
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for LifetimeScope {
    fn layout() -> ir::LayoutIr {
        LifetimeId::layout()
    }

    fn lexical_id() -> LexicalId {
        LifetimeId::lexical_id()
    }

    fn add_references(references: &mut References) {
        LifetimeId::add_references(references)
    }
}

/// Id of a scope's lifetime.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[repr(transparent)]
pub struct LifetimeId(pub ObjectId);

impl LifetimeId {
    /// Nil `LifetimeId` (all zeros).
    pub const NIL: Self = Self(ObjectId::NIL);

    /// Binds the id to a client and creates a [`Lifetime`].
    pub async fn bind(self, client: &Handle) -> Result<Lifetime, Error> {
        Lifetime::new(client, self).await
    }

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(self) -> bool {
        self.0.is_nil()
    }
}

impl PrimaryTag for LifetimeId {
    type Tag = tags::ObjectId;
}

impl Serialize<tags::ObjectId> for LifetimeId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(self.0)
    }
}

impl Serialize<tags::ObjectId> for &LifetimeId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<tags::ObjectId> for LifetimeId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize().map(Self)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for LifetimeId {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Lifetime.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::LIFETIME
    }

    fn add_references(_references: &mut References) {}
}

impl From<ObjectId> for LifetimeId {
    fn from(id: ObjectId) -> Self {
        Self(id)
    }
}

impl From<LifetimeId> for ObjectId {
    fn from(id: LifetimeId) -> Self {
        id.0
    }
}

/// Notifies when a [`LifetimeScope`] ends.
///
/// See [`LifetimeScope`] for information and usages examples.
#[derive(Debug)]
pub struct Lifetime {
    listener: Option<LifetimeListener>,
    id: LifetimeId,
    found: bool,
}

impl Lifetime {
    /// Create a `Lifetime` from an id.
    pub async fn new(client: &Handle, id: LifetimeId) -> Result<Self, Error> {
        let listener = client.create_lifetime_listener().await?;
        listener.start(id.0.uuid).await?;

        Ok(Self {
            listener: Some(listener),
            id,
            found: false,
        })
    }

    /// Returns the lifetime's id.
    pub fn id(&self) -> LifetimeId {
        self.id
    }

    /// Returns a handle to the client that was used to create the lifetime.
    ///
    /// `None` will be returned after [`poll_ended`](Self::poll_ended) or [`ended`](Self::ended) has
    /// resolved.
    pub fn client(&self) -> Option<&Handle> {
        self.listener.as_ref().map(LifetimeListener::client)
    }

    /// Poll whether the associated scope has ended.
    pub fn poll_ended(&mut self, cx: &mut Context) -> Poll<()> {
        let Some(ref mut listener) = self.listener else {
            return Poll::Ready(());
        };

        loop {
            let event = match listener.poll_next_event(cx) {
                Poll::Ready(Some(event)) => event,
                Poll::Ready(None) => break,
                Poll::Pending => return Poll::Pending,
            };

            match event {
                BusListenerEvent::Started(_) => {}

                BusListenerEvent::Event(BusEvent::ObjectCreated(id)) => {
                    debug_assert_eq!(id.uuid, self.id.0.uuid);

                    if id.cookie == self.id.0.cookie {
                        self.found = true;
                    } else {
                        break;
                    }
                }

                BusListenerEvent::Event(BusEvent::ObjectDestroyed(id)) => {
                    debug_assert_eq!(id.uuid, self.id.0.uuid);
                    break;
                }

                BusListenerEvent::CurrentFinished => {
                    if !self.found {
                        break;
                    }
                }

                BusListenerEvent::Stopped
                | BusListenerEvent::Event(BusEvent::ServiceCreated(_))
                | BusListenerEvent::Event(BusEvent::ServiceDestroyed(_)) => unreachable!(),
            }
        }

        self.listener = None;
        Poll::Ready(())
    }

    /// Await the associated scope to end.
    pub async fn ended(&mut self) {
        future::poll_fn(|cx| self.poll_ended(cx)).await
    }

    /// Check if the associated scope has ended.
    ///
    /// This will only return true after [`poll_ended`](Self::poll_ended) has returned
    /// `Poll::Ready(())` or [`ended`](Self::ended) has been awaited.
    pub fn has_ended(&self) -> bool {
        self.listener.is_none()
    }
}

impl Future for Lifetime {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        self.poll_ended(cx)
    }
}

impl FusedFuture for Lifetime {
    fn is_terminated(&self) -> bool {
        self.has_ended()
    }
}

#[derive(Debug)]
pub(crate) struct LifetimeListener {
    cookie: BusListenerCookie,
    client: Handle,
    events: UnboundedReceiver<BusListenerEvent>,
}

impl LifetimeListener {
    pub(crate) fn new(
        cookie: BusListenerCookie,
        client: Handle,
        events: UnboundedReceiver<BusListenerEvent>,
    ) -> Self {
        Self {
            cookie,
            client,
            events,
        }
    }

    async fn start(&self, uuid: ObjectUuid) -> Result<(), Error> {
        self.client
            .add_bus_listener_filter(self.cookie, BusListenerFilter::object(uuid))?;

        self.client
            .start_bus_listener(self.cookie, BusListenerScope::All)
            .await?;

        Ok(())
    }

    fn client(&self) -> &Handle {
        &self.client
    }

    fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<BusListenerEvent>> {
        Pin::new(&mut self.events).poll_next(cx)
    }
}

impl Drop for LifetimeListener {
    fn drop(&mut self) {
        self.client.destroy_bus_listener_now(self.cookie);
    }
}
