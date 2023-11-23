#[cfg(test)]
mod test;

use crate::bus_listener::BusListener;
use crate::error::Error;
use crate::handle::Handle;
use aldrin_proto::{
    BusEvent, BusListenerFilter, BusListenerScope, ObjectCookie, ObjectId, ObjectUuid,
    ServiceCookie, ServiceId, ServiceUuid,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::future;
use std::mem;
use std::task::{Context, Poll};

/// Discovers objects with multiple services on the bus.
///
/// `Discover`s are similar to [`BusListener`s](BusListener), in that they watch the bus for objects
/// and services, and emit events in certain situations. The key difference is, that they focus on
/// objects with specific sets of services. They then emit only one event, that gives access to all
/// related IDs. A `BusListener`s on the other hand, would emit multiple events, one for the object
/// and one per service.
///
/// The set of objects (and associated services) that a `Discoverer` looks for is configured in
/// advance through a [`DiscovererBuilder`], which can be created directly from a [`Handle`]:
///
/// ```
/// # use aldrin_client::{Discoverer, DiscovererBuilder};
/// # use aldrin_test::tokio::TestBroker;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = TestBroker::new();
/// # let handle = broker.add_client().await;
/// # fn k(_: &DiscovererBuilder<()>) {}
/// let builder = Discoverer::builder(&handle);
/// # k(&builder);
/// let builder = DiscovererBuilder::new(&handle); // Alternative 1
/// # k(&builder);
/// let builder = handle.create_discoverer(); // Alternative 2
/// # k(&builder);
/// # Ok(())
/// # }
/// ```
///
/// When configuring objects, you must choose whether the `Discoverer` matches only on a specific
/// [`ObjectUuid`] or not. Set an `ObjectUuid` when you are looking for a singleton on the bus. Do
/// not set an `ObjectUuid` when you are looking for potentially multiple objects with the same set
/// of services.
///
/// You can configure arbitrarily many objects. To help distinguish them when events are emitted,
/// the `Discoverer` associates each object with a key. [`DiscovererEvent`s](DiscovererEvent) then
/// give access to the key they are related to. Good candidates for keys are either integers or
/// custom `enum`s.
///
/// In the following example, a discoverer is configured for 2 different kinds of objects. One
/// specific object with a fixed UUID and 2 services. And second, a class of objects with just 1
/// service.
///
/// ```
/// # use aldrin_client::{Discoverer, DiscovererEventKind};
/// # use aldrin_proto::{ObjectUuid, ServiceUuid};
/// # use aldrin_test::tokio::TestBroker;
/// # use uuid::uuid;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = TestBroker::new();
/// # let handle = broker.add_client().await;
/// const OBJECT_UUID: ObjectUuid = ObjectUuid(uuid!("730c1d68-212b-4181-9813-811948813809"));
/// const SERVICE_UUID_1: ServiceUuid = ServiceUuid(uuid!("25b952af-7447-4275-9a68-1f9b689d96a4"));
/// const SERVICE_UUID_2: ServiceUuid = ServiceUuid(uuid!("5456b1f9-5c2e-46b0-b0d7-bad82cbc957b"));
///
/// let mut discoverer = Discoverer::builder(&handle)
///     .add_object(1, Some(OBJECT_UUID), [SERVICE_UUID_1, SERVICE_UUID_2])
///     .add_object(2, None, [SERVICE_UUID_1])
///     .build()
///     .await?;
///
/// let mut obj = handle.create_object(OBJECT_UUID).await?;
/// let svc1 = obj.create_service(SERVICE_UUID_1, 0).await?;
///
/// // At this point, `obj` satisfies the requirements of the object configured with the key 2.
/// let ev = discoverer.next_event().await.unwrap();
/// assert_eq!(*ev.key(), 2);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_1), svc1.id());
///
/// let svc2 = obj.create_service(SERVICE_UUID_2, 0).await?;
///
/// // Now `obj` completes the requirements the object configured with the key 1.
/// let ev = discoverer.next_event().await.unwrap();
/// assert_eq!(*ev.key(), 1);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_1), svc1.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_2), svc2.id());
///
/// # discoverer.stop().await?;
/// # assert!(discoverer.next_event().await.is_none());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Discoverer<Key> {
    bus_listener: BusListener,
    specific: Vec<SpecificObject<Key>>,
    any: Vec<AnyObject<Key>>,
    pending_events: VecDeque<PendingEvent>,
}

impl<Key> Discoverer<Key> {
    /// Create a builder for a `Discoverer`.
    pub fn builder(client: &Handle) -> DiscovererBuilder<Key> {
        DiscovererBuilder::new(client)
    }

    async fn new(
        client: &Handle,
        specific: Vec<SpecificObject<Key>>,
        any: Vec<AnyObject<Key>>,
    ) -> Result<Self, Error> {
        let mut bus_listener = client.create_bus_listener().await?;

        for specific in &specific {
            bus_listener.add_filter(BusListenerFilter::object(specific.uuid()))?;

            for service in specific.services() {
                bus_listener.add_filter(BusListenerFilter::specific_object_and_service(
                    specific.uuid(),
                    service,
                ))?;
            }
        }

        for any in &any {
            let mut empty = true;

            for service in any.services() {
                bus_listener.add_filter(BusListenerFilter::any_object_specific_service(service))?;
                empty = false;
            }

            if empty {
                bus_listener.add_filter(BusListenerFilter::any_object())?;
            }
        }

        let mut this = Self {
            bus_listener,
            specific,
            any,
            pending_events: VecDeque::new(),
        };

        this.start().await?;
        Ok(this)
    }

    /// Starts the discoverer after it has been stopped.
    ///
    /// `Discoverer`s are initially started already. This function fails if trying to start a
    /// discoverer that isn't stopped.
    pub async fn start(&mut self) -> Result<(), Error> {
        self.bus_listener.start(BusListenerScope::All).await
    }

    /// Stops the discoverer.
    ///
    /// `Discoverer`s always begin started. Stopping a discoverer will cause
    /// [`next_event`](Self::next_event) to return `None` after all enqueued events have been
    /// drained.
    ///
    /// Be very careful with function, as the discoverer might miss events from the broker and its
    /// internal state might become invalid. There should normally not be any reason to stop a
    /// discoverer.
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.bus_listener.stop().await
    }

    /// Poll the discoverer for an event.
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<DiscovererEvent<'_, Key>>> {
        if self.specific.is_empty() && self.any.is_empty() {
            return Poll::Ready(None);
        }

        loop {
            if let Some(event) = self.pending_events.pop_front() {
                return Poll::Ready(Some(DiscovererEvent::new(
                    self,
                    event.object_type,
                    event.index,
                    event.kind,
                    event.object_id,
                )));
            }

            let event = match self.bus_listener.poll_next_event(cx) {
                Poll::Ready(Some(event)) => event,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            for (i, specific) in self.specific.iter_mut().enumerate() {
                if let Some((kind, object_id)) = specific.handle_bus_event(event) {
                    self.pending_events.push_back(PendingEvent::new(
                        ObjectType::Specific,
                        i,
                        kind,
                        object_id,
                    ));
                }
            }

            for (i, any) in self.any.iter_mut().enumerate() {
                if let Some((kind, object_id)) = any.handle_bus_event(event) {
                    self.pending_events.push_back(PendingEvent::new(
                        ObjectType::Any,
                        i,
                        kind,
                        object_id,
                    ));
                }
            }
        }
    }

    /// Await an event from the discoverer.
    pub async fn next_event(&mut self) -> Option<DiscovererEvent<'_, Key>> {
        future::poll_fn(|cx| match self.poll_next_event(cx) {
            // SAFETY: Extend the lifetime in the event such that it borrows the Discoverer
            // directly, instead of this closure. This cannot lead to multiple mutable borrows,
            // because (1) the PollFn is dropped after the await point in this function, and (2) the
            // future generated by this async fn panics if polled again after completion.
            Poll::Ready(Some(event)) => unsafe {
                Poll::Ready(Some(mem::transmute::<
                    DiscovererEvent<'_, Key>,
                    DiscovererEvent<'_, Key>,
                >(event)))
            },

            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        })
        .await
    }

    fn key(&self, object_type: ObjectType, index: usize) -> &Key {
        match object_type {
            ObjectType::Specific => self.specific[index].key(),
            ObjectType::Any => self.any[index].key(),
        }
    }

    fn service_id(
        &self,
        object_type: ObjectType,
        index: usize,
        object: ObjectUuid,
        service: ServiceUuid,
    ) -> Option<ServiceId> {
        match object_type {
            ObjectType::Specific => self.specific[index].service_id(service),
            ObjectType::Any => self.any[index].service_id(object, service),
        }
    }
}

/// Builder for `Discoverer`s.
///
/// See [`Discoverer`] for usage examples.
#[derive(Debug)]
pub struct DiscovererBuilder<'a, Key> {
    client: &'a Handle,
    specific: Vec<SpecificObject<Key>>,
    any: Vec<AnyObject<Key>>,
}

impl<'a, Key> DiscovererBuilder<'a, Key> {
    /// Creates a new `DiscovererBuilder`.
    pub fn new(client: &'a Handle) -> Self {
        Self {
            client,
            specific: Vec::new(),
            any: Vec::new(),
        }
    }

    /// Builds the discoverer with the configured set of objects.
    pub async fn build(self) -> Result<Discoverer<Key>, Error> {
        Discoverer::new(self.client, self.specific, self.any).await
    }

    /// Add an object to the discoverer.
    ///
    /// The `key` is an arbitrary value that can later be queried again on
    /// [`DiscovererEvent`s](DiscovererEvent). It can be used to distinguish events when more than
    /// one object has been added.
    ///
    /// When specifying an [`ObjectUuid`], the discoverer will match only on that UUID. Otherwise,
    /// the discoverer will emit events for every object that matches the set of services.
    pub fn add_object(
        mut self,
        key: Key,
        object: Option<ObjectUuid>,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        if let Some(object) = object {
            self.specific
                .push(SpecificObject::new(key, object, services));
        } else {
            self.any.push(AnyObject::new(key, services));
        }

        self
    }
}

#[derive(Debug)]
struct SpecificObject<Key> {
    key: Key,
    uuid: ObjectUuid,
    cookie: Option<ObjectCookie>,
    services: HashMap<ServiceUuid, Option<ServiceCookie>>,
    created: bool,
}

impl<Key> SpecificObject<Key> {
    fn new(key: Key, uuid: ObjectUuid, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        Self {
            key,
            uuid,
            cookie: None,
            services: services.into_iter().map(|uuid| (uuid, None)).collect(),
            created: false,
        }
    }

    fn key(&self) -> &Key {
        &self.key
    }

    fn uuid(&self) -> ObjectUuid {
        self.uuid
    }

    fn services(&self) -> impl Iterator<Item = ServiceUuid> + '_ {
        self.services.keys().copied()
    }

    fn service_id(&self, uuid: ServiceUuid) -> Option<ServiceId> {
        let service_cookie = (*self.services.get(&uuid)?)?;
        let object_cookie = self.cookie?;

        Some(ServiceId::new(
            ObjectId::new(self.uuid, object_cookie),
            uuid,
            service_cookie,
        ))
    }

    fn handle_bus_event(&mut self, event: BusEvent) -> Option<(DiscovererEventKind, ObjectId)> {
        match event {
            BusEvent::ObjectCreated(id) => self.object_created(id),
            BusEvent::ObjectDestroyed(id) => self.object_destroyed(id),
            BusEvent::ServiceCreated(id) => self.service_created(id),
            BusEvent::ServiceDestroyed(id) => self.service_destroyed(id),
        }
    }

    fn object_created(&mut self, id: ObjectId) -> Option<(DiscovererEventKind, ObjectId)> {
        if id.uuid != self.uuid {
            return None;
        }

        debug_assert!(self.cookie.is_none());
        debug_assert!(self.services.values().all(Option::is_none));
        debug_assert!(!self.created);

        self.cookie = Some(id.cookie);

        if self.services.is_empty() {
            self.created = true;
            Some((DiscovererEventKind::Created, id))
        } else {
            None
        }
    }

    fn object_destroyed(&mut self, id: ObjectId) -> Option<(DiscovererEventKind, ObjectId)> {
        if id.uuid != self.uuid {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.cookie));
        debug_assert!(self.services.values().all(Option::is_none));

        self.cookie = None;

        if self.created {
            self.created = false;
            Some((DiscovererEventKind::Destroyed, id))
        } else {
            None
        }
    }

    fn service_created(&mut self, id: ServiceId) -> Option<(DiscovererEventKind, ObjectId)> {
        if id.object_id.uuid != self.uuid {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.object_id.cookie));

        let Some(service) = self.services.get_mut(&id.uuid) else {
            return None;
        };

        debug_assert!(service.is_none());
        debug_assert!(!self.created);

        *service = Some(id.cookie);

        if self.services.values().all(Option::is_some) {
            self.created = true;
            Some((DiscovererEventKind::Created, id.object_id))
        } else {
            None
        }
    }

    fn service_destroyed(&mut self, id: ServiceId) -> Option<(DiscovererEventKind, ObjectId)> {
        if id.object_id.uuid != self.uuid {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.object_id.cookie));

        let Some(service) = self.services.get_mut(&id.uuid) else {
            return None;
        };

        debug_assert_eq!(*service, Some(id.cookie));

        *service = None;

        if self.created {
            self.created = false;
            Some((DiscovererEventKind::Destroyed, id.object_id))
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AnyObject<Key> {
    key: Key,
    services: HashMap<ServiceUuid, HashMap<ObjectUuid, AnyObjectCookies>>,
    created: HashSet<ObjectId>,
}

impl<Key> AnyObject<Key> {
    fn new(key: Key, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        Self {
            key,
            services: services
                .into_iter()
                .map(|uuid| (uuid, HashMap::new()))
                .collect(),
            created: HashSet::new(),
        }
    }

    fn key(&self) -> &Key {
        &self.key
    }

    fn services(&self) -> impl Iterator<Item = ServiceUuid> + '_ {
        self.services.keys().copied()
    }

    fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        let cookies = self.services.get(&service)?.get(&object)?;

        Some(ServiceId::new(
            ObjectId::new(object, cookies.object),
            service,
            cookies.service,
        ))
    }

    fn handle_bus_event(&mut self, event: BusEvent) -> Option<(DiscovererEventKind, ObjectId)> {
        match event {
            BusEvent::ObjectCreated(id) => self.object_created(id),
            BusEvent::ObjectDestroyed(id) => self.object_destroyed(id),
            BusEvent::ServiceCreated(id) => self.service_created(id),
            BusEvent::ServiceDestroyed(id) => self.service_destroyed(id),
        }
    }

    fn object_created(&mut self, id: ObjectId) -> Option<(DiscovererEventKind, ObjectId)> {
        if self.services.is_empty() {
            let inserted = self.created.insert(id);
            debug_assert!(inserted);
            Some((DiscovererEventKind::Created, id))
        } else {
            None
        }
    }

    fn object_destroyed(&mut self, id: ObjectId) -> Option<(DiscovererEventKind, ObjectId)> {
        if self.created.remove(&id) {
            Some((DiscovererEventKind::Destroyed, id))
        } else {
            None
        }
    }

    fn service_created(&mut self, id: ServiceId) -> Option<(DiscovererEventKind, ObjectId)> {
        let Some(service) = self.services.get_mut(&id.uuid) else {
            return None;
        };

        let dup = service.insert(id.object_id.uuid, AnyObjectCookies::new(id));
        debug_assert!(dup.is_none());

        if self
            .services
            .values()
            .all(|c| c.contains_key(&id.object_id.uuid))
        {
            let inserted = self.created.insert(id.object_id);
            debug_assert!(inserted);
            Some((DiscovererEventKind::Created, id.object_id))
        } else {
            None
        }
    }

    fn service_destroyed(&mut self, id: ServiceId) -> Option<(DiscovererEventKind, ObjectId)> {
        let Some(service) = self.services.get_mut(&id.uuid) else {
            return None;
        };

        let cookies = service.remove(&id.object_id.uuid);
        debug_assert!(cookies.is_some());

        if self.created.remove(&id.object_id) {
            Some((DiscovererEventKind::Destroyed, id.object_id))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AnyObjectCookies {
    object: ObjectCookie,
    service: ServiceCookie,
}

impl AnyObjectCookies {
    fn new(id: ServiceId) -> Self {
        Self {
            object: id.object_id.cookie,
            service: id.cookie,
        }
    }
}

/// Event emitted by `Discoverer`s.
#[derive(Debug)]
pub struct DiscovererEvent<'a, Key> {
    discoverer: &'a Discoverer<Key>,
    object_type: ObjectType,
    index: usize,
    kind: DiscovererEventKind,
    object_id: ObjectId,
}

impl<'a, Key> DiscovererEvent<'a, Key> {
    fn new(
        discoverer: &'a Discoverer<Key>,
        object_type: ObjectType,
        index: usize,
        kind: DiscovererEventKind,
        object_id: ObjectId,
    ) -> Self {
        Self {
            discoverer,
            object_type,
            index,
            kind,
            object_id,
        }
    }

    /// Specifies whether the object was created or destroyed.
    pub fn kind(&self) -> DiscovererEventKind {
        self.kind
    }

    /// Returns a references to the key associated with this object.
    pub fn key(&self) -> &Key {
        self.discoverer.key(self.object_type, self.index)
    }

    /// Returns the object ID that prompted this event.
    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }

    /// Returns a service ID that is owned by this event's object.
    ///
    /// This function can only be called for those events that are emitted when an object is created
    /// ([`kind`](Self::kind) returns [`DiscovererEventKind::Created`]). It will panic otherwise.
    ///
    /// This function will also panic if `uuid` is not one of the UUIDs specified when
    /// [`add_object`](DiscovererBuilder::add_object) was called.
    pub fn service_id(&self, uuid: ServiceUuid) -> ServiceId {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        let id = self
            .discoverer
            .service_id(self.object_type, self.index, self.object_id.uuid, uuid)
            .expect("invalid UUID");

        debug_assert_eq!(id.uuid, uuid);
        debug_assert_eq!(id.object_id, self.object_id);

        id
    }
}

/// Specifies whether an object was created or destroyed.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiscovererEventKind {
    /// An object was created.
    Created,

    /// An object was destroyed.
    Destroyed,
}

#[derive(Debug, Copy, Clone)]
enum ObjectType {
    Specific,
    Any,
}

#[derive(Debug)]
struct PendingEvent {
    object_type: ObjectType,
    index: usize,
    kind: DiscovererEventKind,
    object_id: ObjectId,
}

impl PendingEvent {
    fn new(
        object_type: ObjectType,
        index: usize,
        kind: DiscovererEventKind,
        object_id: ObjectId,
    ) -> Self {
        Self {
            object_type,
            index,
            kind,
            object_id,
        }
    }
}
