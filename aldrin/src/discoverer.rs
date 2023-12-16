#[cfg(test)]
mod test;

use crate::bus_listener::BusListener;
use crate::core::{
    BusEvent, BusListenerFilter, BusListenerScope, ObjectCookie, ObjectId, ObjectUuid,
    ServiceCookie, ServiceId, ServiceUuid,
};
use crate::error::Error;
use crate::handle::Handle;
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
/// # use aldrin::{Discoverer, DiscovererBuilder};
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
/// [`ObjectUuid`] or not. Set an `ObjectUuid` when you are looking for a single specific object on
/// the bus. Do not set an `ObjectUuid` when you are looking for potentially multiple objects with
/// the same set of services.
///
/// You can configure arbitrarily many objects. To help distinguish them when events are emitted,
/// the `Discoverer` associates each object with a key. [`DiscovererEventRef`s](DiscovererEventRef)
/// then give access to the key they are related to. Good candidates for keys are either integers or
/// custom `enum`s.
///
/// In the following example, a discoverer is configured for 2 different kinds of objects. One
/// specific object with a fixed UUID and 2 services. And second, a class of objects with just 1
/// service.
///
/// ```
/// # use aldrin::{Discoverer, DiscovererEventKind};
/// # use aldrin::core::{ObjectUuid, ServiceUuid};
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
///     .specific(1, OBJECT_UUID, [SERVICE_UUID_1, SERVICE_UUID_2])
///     .any(2, [SERVICE_UUID_1])
///     .build()
///     .await?;
///
/// let mut obj = handle.create_object(OBJECT_UUID).await?;
/// let svc1 = obj.create_service(SERVICE_UUID_1, 0).await?;
///
/// // At this point, `obj` satisfies the requirements of the object configured with the key 2.
/// let ev = discoverer.next_event_ref().await.unwrap();
/// assert_eq!(*ev.key(), 2);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_1), svc1.id());
///
/// let svc2 = obj.create_service(SERVICE_UUID_2, 0).await?;
///
/// // Now `obj` completes the requirements the object configured with the key 1.
/// let ev = discoverer.next_event_ref().await.unwrap();
/// assert_eq!(*ev.key(), 1);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_1), svc1.id());
/// assert_eq!(ev.service_id(SERVICE_UUID_2), svc2.id());
///
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
        current_only: bool,
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

        if current_only {
            this.bus_listener.start(BusListenerScope::Current).await?;
        } else {
            this.bus_listener.start(BusListenerScope::All).await?;
        }

        Ok(this)
    }

    /// Returns a handle to the client that was used to create the discoverer.
    pub fn client(&self) -> &Handle {
        self.bus_listener.client()
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.bus_listener.stop().await?;
        while self.next_event_ref().await.is_some() {}

        for specific in &mut self.specific {
            specific.reset();
        }

        for any in &mut self.any {
            any.reset();
        }

        self.pending_events.clear();
        Ok(())
    }

    /// Restarts the discoverer.
    ///
    /// All pending events will be discarded. The discoverer will be configured to consider all
    /// objects and services on the bus, as if it was built again with [`DiscovererBuilder::build`].
    pub async fn restart(&mut self) -> Result<(), Error> {
        self.stop().await?;
        self.bus_listener.start(BusListenerScope::All).await?;
        Ok(())
    }

    /// Restarts the discoverer and configures it to consider only current objects and services.
    ///
    /// All pending events will be discarded. The discoverer will be configured to consider only
    /// current objects and services on the bus, as if it was built again with
    /// [`DiscovererBuilder::build_current_only`].
    pub async fn restart_current_only(&mut self) -> Result<(), Error> {
        self.stop().await?;
        self.bus_listener.start(BusListenerScope::Current).await?;
        Ok(())
    }

    /// Indicates whether the discoverer can return more events.
    ///
    /// Discoverers can only finish if they are considering only current objects and services,
    /// i.e. built with [`build_current_only`](DiscovererBuilder::build_current_only) or restarted
    /// with [`restart_current_only`](Self::restart_current_only`).
    pub fn is_finished(&self) -> bool {
        if self.specific.is_empty() && self.any.is_empty() {
            true
        } else {
            self.bus_listener.is_finished()
        }
    }

    /// Poll the discoverer for an event.
    pub fn poll_next_event_ref(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Option<DiscovererEventRef<'_, Key>>> {
        if self.specific.is_empty() && self.any.is_empty() {
            return Poll::Ready(None);
        }

        loop {
            if let Some(event) = self.pending_events.pop_front() {
                return Poll::Ready(Some(DiscovererEventRef::new(
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
    pub async fn next_event_ref(&mut self) -> Option<DiscovererEventRef<'_, Key>> {
        future::poll_fn(|cx| match self.poll_next_event_ref(cx) {
            // SAFETY: Extend the lifetime in the event such that it borrows the Discoverer
            // directly, instead of this closure. This cannot lead to multiple mutable borrows,
            // because (1) the PollFn is dropped after the await point in this function, and (2) the
            // future generated by this async fn panics if polled again after completion.
            Poll::Ready(Some(event)) => unsafe {
                Poll::Ready(Some(mem::transmute::<
                    DiscovererEventRef<'_, Key>,
                    DiscovererEventRef<'_, Key>,
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

    fn service_cookie(
        &self,
        object_type: ObjectType,
        index: usize,
        object: ObjectUuid,
        service: ServiceUuid,
    ) -> Option<ServiceCookie> {
        match object_type {
            ObjectType::Specific => self.specific[index].service_cookie(service),
            ObjectType::Any => self.any[index].service_cookie(object, service),
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
        Discoverer::new(self.client, self.specific, self.any, false).await
    }

    /// Builds the discoverer and configures it to consider only current objects and services.
    ///
    /// Unlike [`build`](Self::build), the discoverer will consider only those objects and services
    /// that exist already on the bus.
    pub async fn build_current_only(self) -> Result<Discoverer<Key>, Error> {
        Discoverer::new(self.client, self.specific, self.any, true).await
    }

    /// Add an object to the discoverer.
    ///
    /// The `key` is an arbitrary value that can later be queried again on
    /// [`DiscovererEventRef`s](DiscovererEventRef). It can be used to distinguish events when more
    /// than one object has been added.
    ///
    /// When specifying an [`ObjectUuid`], the discoverer will match only on that UUID. Otherwise,
    /// the discoverer will emit events for every object that matches the set of services.
    pub fn object(
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

    /// Registers interest in any object implementing a set of services.
    ///
    /// This is a shorthand for calling `object(key, None, services)`.
    pub fn any(self, key: Key, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        self.object(key, None, services)
    }

    /// Registers interest in a specific object implementing a set of services.
    ///
    /// This is a shorthand for calling `object(key, Some(object), services)`.
    pub fn specific(
        self,
        key: Key,
        object: impl Into<ObjectUuid>,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        self.object(key, Some(object.into()), services)
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

    fn service_cookie(&self, uuid: ServiceUuid) -> Option<ServiceCookie> {
        self.services.get(&uuid).copied().flatten()
    }

    fn reset(&mut self) {
        self.cookie = None;
        self.created = false;

        for cookie in self.services.values_mut() {
            *cookie = None;
        }
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

        let service = self.services.get_mut(&id.uuid)?;

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

        let service = self.services.get_mut(&id.uuid)?;

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
    services: HashMap<ServiceUuid, HashMap<ObjectUuid, ServiceCookie>>,
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

    fn service_cookie(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceCookie> {
        self.services.get(&service)?.get(&object).copied()
    }

    fn reset(&mut self) {
        self.created.clear();

        for cookies in self.services.values_mut() {
            cookies.clear();
        }
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
        let service = self.services.get_mut(&id.uuid)?;
        let dup = service.insert(id.object_id.uuid, id.cookie);
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
        let service = self.services.get_mut(&id.uuid)?;
        let cookies = service.remove(&id.object_id.uuid);
        debug_assert!(cookies.is_some());

        if self.created.remove(&id.object_id) {
            Some((DiscovererEventKind::Destroyed, id.object_id))
        } else {
            None
        }
    }
}

/// Event emitted by `Discoverer`s.
#[derive(Debug)]
pub struct DiscovererEventRef<'a, Key> {
    discoverer: &'a Discoverer<Key>,
    object_type: ObjectType,
    index: usize,
    kind: DiscovererEventKind,
    object_id: ObjectId,
}

impl<'a, Key> DiscovererEventRef<'a, Key> {
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
    /// [`object`](DiscovererBuilder::object) was called.
    pub fn service_id(&self, uuid: ServiceUuid) -> ServiceId {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        let cookie = self
            .discoverer
            .service_cookie(self.object_type, self.index, self.object_id.uuid, uuid)
            .expect("invalid UUID");

        ServiceId::new(self.object_id, uuid, cookie)
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
