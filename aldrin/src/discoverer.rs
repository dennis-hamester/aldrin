#[cfg(test)]
mod test;

use crate::bus_listener::BusListener;
use crate::core::{
    BusEvent, BusListenerFilter, BusListenerScope, ObjectCookie, ObjectId, ObjectUuid,
    ServiceCookie, ServiceId, ServiceUuid,
};
use crate::error::Error;
use crate::handle::Handle;
use futures_core::stream::{FusedStream, Stream};
use std::collections::hash_map::{self, HashMap};
use std::collections::VecDeque;
use std::hash::Hash;
use std::iter::{FlatMap, FusedIterator};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{future, option};

/// Discovers objects with multiple services on the bus.
///
/// `Discover`s are similar to [`BusListener`s](BusListener), in that they watch the bus for objects
/// and services, and emit events in certain situations. The key difference is, that they focus on
/// objects with specific sets of services. They then emit only one event, that gives access to all
/// related ids. A `BusListener`s on the other hand, would emit multiple events, one for the object
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
/// # let mut broker = TestBroker::new();
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
/// the `Discoverer` associates each object with a key. [`DiscovererEvent`s](DiscovererEvent) then
/// give access to the key they are related to. Good candidates for keys are either integers or
/// custom `enum`s.
///
/// In the following example, a discoverer is configured for 2 different kinds of objects. One
/// specific object with a fixed UUID and 2 services. And second, a class of objects with just 1
/// service.
///
/// ```
/// # use aldrin::{Discoverer, DiscovererEventKind};
/// # use aldrin::core::{ObjectUuid, ServiceUuid};
/// # use aldrin::low_level::ServiceInfo;
/// # use aldrin_test::tokio::TestBroker;
/// # use uuid::uuid;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut broker = TestBroker::new();
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
/// let info = ServiceInfo::new(0);
/// let svc1 = obj.create_service(SERVICE_UUID_1, info).await?;
///
/// // At this point, `obj` satisfies the requirements of the object configured with the key 2.
/// let ev = discoverer.next_event().await.unwrap();
/// assert_eq!(ev.key(), 2);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(&discoverer, SERVICE_UUID_1), svc1.id());
///
/// let svc2 = obj.create_service(SERVICE_UUID_2, info).await?;
///
/// // Now `obj` completes the requirements the object configured with the key 1.
/// let ev = discoverer.next_event().await.unwrap();
/// assert_eq!(ev.key(), 1);
/// assert_eq!(ev.kind(), DiscovererEventKind::Created);
/// assert_eq!(ev.object_id(), obj.id());
/// assert_eq!(ev.service_id(&discoverer, SERVICE_UUID_1), svc1.id());
/// assert_eq!(ev.service_id(&discoverer, SERVICE_UUID_2), svc2.id());
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Discoverer<Key> {
    listener: BusListener,
    entries: HashMap<Key, DiscovererEntry<Key>>,
    events: VecDeque<DiscovererEvent<Key>>,
}

impl<Key> Discoverer<Key>
where
    Key: Copy + Eq + Hash,
{
    async fn new(
        client: &Handle,
        entries: HashMap<Key, DiscovererEntry<Key>>,
        current_only: bool,
    ) -> Result<Self, Error> {
        let mut listener = client.create_bus_listener().await?;

        for entry in entries.values() {
            entry.add_filter(&mut listener)?;
        }

        if current_only {
            listener.start(BusListenerScope::Current).await?;
        } else {
            listener.start(BusListenerScope::All).await?;
        }

        Ok(Self {
            listener,
            entries,
            events: VecDeque::new(),
        })
    }

    /// Create a builder for a `Discoverer`.
    pub fn builder(client: &Handle) -> DiscovererBuilder<Key> {
        DiscovererBuilder::new(client)
    }

    /// Returns a handle to the client that was used to create the discoverer.
    pub fn client(&self) -> &Handle {
        self.listener.client()
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.listener.stop().await?;
        while self.next_event().await.is_some() {}

        for entry in self.entries.values_mut() {
            entry.reset();
        }

        self.events.clear();
        Ok(())
    }

    /// Restarts the discoverer.
    ///
    /// All pending events will be discarded. The discoverer will be configured to consider all
    /// objects and services on the bus, as if it was built again with [`DiscovererBuilder::build`].
    pub async fn restart(&mut self) -> Result<(), Error> {
        self.stop().await?;
        self.listener.start(BusListenerScope::All).await?;
        Ok(())
    }

    /// Restarts the discoverer and configures it to consider only current objects and services.
    ///
    /// All pending events will be discarded. The discoverer will be configured to consider only
    /// current objects and services on the bus, as if it was built again with
    /// [`DiscovererBuilder::build_current_only`].
    pub async fn restart_current_only(&mut self) -> Result<(), Error> {
        self.stop().await?;
        self.listener.start(BusListenerScope::Current).await?;
        Ok(())
    }

    /// Indicates whether the discoverer can return more events.
    ///
    /// Discoverers can only finish if they are considering only current objects and services,
    /// i.e. built with [`build_current_only`](DiscovererBuilder::build_current_only) or restarted
    /// with [`restart_current_only`](Self::restart_current_only`).
    pub fn is_finished(&self) -> bool {
        self.entries.is_empty() || self.listener.is_finished()
    }

    /// Queries a specific object id.
    pub fn object_id(&self, key: Key, object: ObjectUuid) -> Option<ObjectId> {
        self.entries
            .get(&key)
            .expect("invalid key")
            .object_id(object)
    }

    /// Queries a specific service id.
    pub fn service_id(
        &self,
        key: Key,
        object: ObjectUuid,
        service: ServiceUuid,
    ) -> Option<ServiceId> {
        self.entries
            .get(&key)
            .expect("invalid key")
            .service_id(object, service)
    }

    /// Returns an entry of the `Discoverer`.
    ///
    /// Entries are directly associated with the keys and correspond to the
    /// [`object`](DiscovererBuilder::object), [`specific`](DiscovererBuilder::specific) and
    /// [`any`](DiscovererBuilder::any) calls on the [`DiscovererBuilder`].
    pub fn entry(&self, key: Key) -> Option<&DiscovererEntry<Key>> {
        self.entries.get(&key)
    }

    /// Returns an iterator over all found objects.
    pub fn iter(&self) -> DiscovererIter<Key> {
        DiscovererIter::new(self.entries.values().flat_map(DiscovererEntry::iter))
    }

    /// Returns an iterator over all found objects corresponding to a specific key.
    pub fn entry_iter(&self, key: Key) -> Option<DiscovererEntryIter<Key>> {
        self.entry(key).map(DiscovererEntry::iter)
    }

    /// Polls the discoverer for an event.
    pub fn poll_next_event(&mut self, cx: &mut Context) -> Poll<Option<DiscovererEvent<Key>>> {
        if self.entries.is_empty() {
            return Poll::Ready(None);
        }

        loop {
            if let Some(event) = self.events.pop_front() {
                return Poll::Ready(Some(event));
            }

            let event = match self.listener.poll_next_event(cx) {
                Poll::Ready(Some(event)) => event,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            for entry in self.entries.values_mut() {
                if let Some(event) = entry.handle_event(event) {
                    self.events.push_back(event);
                }
            }
        }
    }

    /// Awaits an event from the discoverer.
    pub async fn next_event(&mut self) -> Option<DiscovererEvent<Key>> {
        future::poll_fn(|cx| self.poll_next_event(cx)).await
    }
}

impl<Key> Unpin for Discoverer<Key> {}

impl<Key> Stream for Discoverer<Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = DiscovererEvent<Key>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_next_event(cx)
    }
}

impl<Key> FusedStream for Discoverer<Key>
where
    Key: Copy + Eq + Hash,
{
    fn is_terminated(&self) -> bool {
        self.is_finished()
    }
}

impl<'a, Key> IntoIterator for &'a Discoverer<Key>
where
    Key: Copy + Eq + Hash,
{
    type IntoIter = DiscovererIter<'a, Key>;
    type Item = DiscovererIterEntry<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Builder for `Discoverer`s.
///
/// See [`Discoverer`] for usage examples.
#[derive(Debug)]
pub struct DiscovererBuilder<'a, Key> {
    client: &'a Handle,
    entries: HashMap<Key, DiscovererEntry<Key>>,
}

impl<'a, Key> DiscovererBuilder<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    /// Creates a new `DiscovererBuilder`.
    pub fn new(client: &'a Handle) -> Self {
        Self {
            client,
            entries: HashMap::new(),
        }
    }

    /// Builds the discoverer with the configured set of objects.
    pub async fn build(self) -> Result<Discoverer<Key>, Error> {
        Discoverer::new(self.client, self.entries, false).await
    }

    /// Builds the discoverer and configures it to consider only current objects and services.
    ///
    /// Unlike [`build`](Self::build), the discoverer will consider only those objects and services
    /// that exist already on the bus.
    pub async fn build_current_only(self) -> Result<Discoverer<Key>, Error> {
        Discoverer::new(self.client, self.entries, true).await
    }

    /// Add an object to the discoverer.
    ///
    /// The `key` is an arbitrary value that can later be queried again on
    /// [`DiscovererEvent`s](DiscovererEvent). It can be used to distinguish events when more than
    /// one object has been added.
    ///
    /// When specifying an [`ObjectUuid`], the discoverer will match only on that UUID. Otherwise,
    /// the discoverer will emit events for every object that matches the set of services.
    pub fn object(
        self,
        key: Key,
        object: Option<ObjectUuid>,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        match object {
            Some(object) => self.specific(key, object, services),
            None => self.any(key, services),
        }
    }

    /// Registers interest in a specific object implementing a set of services.
    ///
    /// This is a shorthand for calling `object(key, Some(object), services)`.
    pub fn specific(
        mut self,
        key: Key,
        object: impl Into<ObjectUuid>,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        self.entries.insert(
            key,
            SpecificObject::new(key, object.into(), services).into(),
        );

        self
    }

    /// Registers interest in any object implementing a set of services.
    ///
    /// This is a shorthand for calling `object(key, None, services)`.
    pub fn any(mut self, key: Key, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        self.entries
            .insert(key, AnyObject::new(key, services).into());

        self
    }
}

type DiscovererIterInner<'a, Key> = FlatMap<
    hash_map::Values<'a, Key, DiscovererEntry<Key>>,
    DiscovererEntryIter<'a, Key>,
    fn(&'a DiscovererEntry<Key>) -> DiscovererEntryIter<'a, Key>,
>;

/// Iterator over all found objects of a [`Discoverer`].
#[derive(Debug, Clone)]
pub struct DiscovererIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    inner: DiscovererIterInner<'a, Key>,
}

impl<'a, Key> DiscovererIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(inner: DiscovererIterInner<'a, Key>) -> Self {
        Self { inner }
    }
}

impl<'a, Key> Iterator for DiscovererIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = DiscovererIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, Key> FusedIterator for DiscovererIter<'a, Key> where Key: Copy + Eq + Hash {}

/// Entry of a `Discoverer`.
#[derive(Debug)]
pub struct DiscovererEntry<Key> {
    inner: EntryInner<Key>,
}

impl<Key> DiscovererEntry<Key>
where
    Key: Copy + Eq + Hash,
{
    fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.add_filter(listener),
            EntryInner::Any(ref any) => any.add_filter(listener),
        }
    }

    fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match self.inner {
            EntryInner::Specific(ref mut specific) => specific.handle_event(event),
            EntryInner::Any(ref mut any) => any.handle_event(event),
        }
    }

    fn reset(&mut self) {
        match self.inner {
            EntryInner::Specific(ref mut specific) => specific.reset(),
            EntryInner::Any(ref mut any) => any.reset(),
        }
    }

    /// Returns the entries key.
    pub fn key(&self) -> Key {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.key(),
            EntryInner::Any(ref any) => any.key(),
        }
    }

    /// Queries the `ObjectId` of an existing object.
    pub fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.object_id(),
            EntryInner::Any(ref any) => any.object_id(object),
        }
    }

    /// Queries a `ServiceId` of an existing object.
    pub fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.service_id(service),
            EntryInner::Any(ref any) => any.service_id(object, service),
        }
    }

    /// Returns an iterator over all found objects corresponding to this entry.
    pub fn iter(&self) -> DiscovererEntryIter<Key> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.iter().into(),
            EntryInner::Any(ref any) => any.iter().into(),
        }
    }
}

impl<'a, Key> IntoIterator for &'a DiscovererEntry<Key>
where
    Key: Copy + Eq + Hash,
{
    type IntoIter = DiscovererEntryIter<'a, Key>;
    type Item = DiscovererIterEntry<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Key> From<SpecificObject<Key>> for DiscovererEntry<Key> {
    fn from(o: SpecificObject<Key>) -> Self {
        Self {
            inner: EntryInner::Specific(o),
        }
    }
}

impl<Key> From<AnyObject<Key>> for DiscovererEntry<Key> {
    fn from(o: AnyObject<Key>) -> Self {
        Self {
            inner: EntryInner::Any(o),
        }
    }
}

#[derive(Debug)]
enum EntryInner<Key> {
    Specific(SpecificObject<Key>),
    Any(AnyObject<Key>),
}

/// Iterator over all found objects corresponding to a specific key.
#[derive(Debug, Clone)]
pub struct DiscovererEntryIter<'a, Key> {
    inner: EntryIterInner<'a, Key>,
}

impl<'a, Key> Iterator for DiscovererEntryIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = DiscovererIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            EntryIterInner::Specific(ref mut specific) => specific.next().map(Into::into),
            EntryIterInner::Any(ref mut any) => any.next().map(Into::into),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner {
            EntryIterInner::Specific(ref specific) => specific.size_hint(),
            EntryIterInner::Any(ref any) => any.size_hint(),
        }
    }
}

impl<'a, Key> FusedIterator for DiscovererEntryIter<'a, Key> where Key: Copy + Eq + Hash {}

impl<'a, Key> From<SpecificObjectIter<'a, Key>> for DiscovererEntryIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn from(specific: SpecificObjectIter<'a, Key>) -> Self {
        Self {
            inner: EntryIterInner::Specific(specific),
        }
    }
}

impl<'a, Key> From<AnyObjectIter<'a, Key>> for DiscovererEntryIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn from(any: AnyObjectIter<'a, Key>) -> Self {
        Self {
            inner: EntryIterInner::Any(any),
        }
    }
}

#[derive(Debug, Clone)]
enum EntryIterInner<'a, Key> {
    Specific(SpecificObjectIter<'a, Key>),
    Any(AnyObjectIter<'a, Key>),
}

/// Item type when iterating over a [`Discoverer`] or [`DiscovererEntry`].
#[derive(Debug, Copy, Clone)]
pub struct DiscovererIterEntry<'a, Key> {
    inner: IterEntryInner<'a, Key>,
}

impl<'a, Key> DiscovererIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    /// Returns the key corresponding to the object.
    pub fn key(self) -> Key {
        match self.inner {
            IterEntryInner::Specific(specific) => specific.key(),
            IterEntryInner::Any(any) => any.key(),
        }
    }

    /// Returns the object's id.
    pub fn object_id(self) -> ObjectId {
        match self.inner {
            IterEntryInner::Specific(specific) => specific.object_id(),
            IterEntryInner::Any(any) => any.object_id(),
        }
    }

    /// Returns one of the object's service ids.
    pub fn service_id(self, service: ServiceUuid) -> ServiceId {
        match self.inner {
            IterEntryInner::Specific(specific) => specific.service_id(service),
            IterEntryInner::Any(any) => any.service_id(service),
        }
    }
}

impl<'a, Key> From<SpecificObjectIterEntry<'a, Key>> for DiscovererIterEntry<'a, Key> {
    fn from(specific: SpecificObjectIterEntry<'a, Key>) -> Self {
        Self {
            inner: IterEntryInner::Specific(specific),
        }
    }
}

impl<'a, Key> From<AnyObjectIterEntry<'a, Key>> for DiscovererIterEntry<'a, Key> {
    fn from(any: AnyObjectIterEntry<'a, Key>) -> Self {
        Self {
            inner: IterEntryInner::Any(any),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum IterEntryInner<'a, Key> {
    Specific(SpecificObjectIterEntry<'a, Key>),
    Any(AnyObjectIterEntry<'a, Key>),
}

#[derive(Debug)]
struct SpecificObject<Key> {
    key: Key,
    object: ObjectUuid,
    cookie: Option<ObjectCookie>,
    services: HashMap<ServiceUuid, Option<ServiceCookie>>,
    created: bool,
}

impl<Key> SpecificObject<Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(key: Key, object: ObjectUuid, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        Self {
            key,
            object,
            cookie: None,
            services: services.into_iter().map(|s| (s, None)).collect(),
            created: false,
        }
    }

    fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        listener.add_filter(BusListenerFilter::object(self.object))?;

        for service in self.services.keys() {
            listener.add_filter(BusListenerFilter::specific_object_and_service(
                self.object,
                *service,
            ))?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.cookie = None;
        self.created = false;

        for cookie in self.services.values_mut() {
            *cookie = None;
        }
    }

    fn key(&self) -> Key {
        self.key
    }

    fn object_id(&self) -> Option<ObjectId> {
        if self.created {
            Some(ObjectId::new(self.object, self.cookie.unwrap()))
        } else {
            None
        }
    }

    fn service_id(&self, service: ServiceUuid) -> Option<ServiceId> {
        self.object_id().map(|object_id| {
            ServiceId::new(
                object_id,
                service,
                self.services.get(&service).expect("invalid UUID").unwrap(),
            )
        })
    }

    fn iter(&self) -> SpecificObjectIter<Key> {
        if self.created {
            SpecificObjectIter::new(Some(SpecificObjectIterEntry::new(self)))
        } else {
            SpecificObjectIter::new(None)
        }
    }

    fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match event {
            BusEvent::ObjectCreated(id) => self.object_created(id),
            BusEvent::ObjectDestroyed(id) => self.object_destroyed(id),
            BusEvent::ServiceCreated(id) => self.service_created(id),
            BusEvent::ServiceDestroyed(id) => self.service_destroyed(id),
        }
    }

    fn object_created(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if id.uuid != self.object {
            return None;
        }

        debug_assert!(self.cookie.is_none());
        debug_assert!(self.services.values().all(Option::is_none));
        debug_assert!(!self.created);

        self.cookie = Some(id.cookie);

        if self.services.is_empty() {
            self.created = true;

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Created,
                id,
            ))
        } else {
            None
        }
    }

    fn object_destroyed(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if id.uuid != self.object {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.cookie));
        debug_assert!(self.services.values().all(Option::is_none));

        self.cookie = None;

        if self.created {
            self.created = false;

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Destroyed,
                id,
            ))
        } else {
            None
        }
    }

    fn service_created(&mut self, id: ServiceId) -> Option<DiscovererEvent<Key>> {
        if id.object_id.uuid != self.object {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.object_id.cookie));

        let service = self.services.get_mut(&id.uuid)?;

        debug_assert!(service.is_none());
        debug_assert!(!self.created);

        *service = Some(id.cookie);

        if self.services.values().all(Option::is_some) {
            self.created = true;

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Created,
                id.object_id,
            ))
        } else {
            None
        }
    }

    fn service_destroyed(&mut self, id: ServiceId) -> Option<DiscovererEvent<Key>> {
        if id.object_id.uuid != self.object {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.object_id.cookie));

        let service = self.services.get_mut(&id.uuid)?;

        debug_assert_eq!(*service, Some(id.cookie));
        *service = None;

        if self.created {
            self.created = false;

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Destroyed,
                id.object_id,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct SpecificObjectIter<'a, Key> {
    next: option::IntoIter<SpecificObjectIterEntry<'a, Key>>,
}

impl<'a, Key> SpecificObjectIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(next: Option<SpecificObjectIterEntry<'a, Key>>) -> Self {
        Self {
            next: next.into_iter(),
        }
    }
}

impl<'a, Key> Iterator for SpecificObjectIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = SpecificObjectIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.next.size_hint()
    }
}

#[derive(Debug, Copy, Clone)]
struct SpecificObjectIterEntry<'a, Key> {
    object: &'a SpecificObject<Key>,
}

impl<'a, Key> SpecificObjectIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(object: &'a SpecificObject<Key>) -> Self {
        Self { object }
    }

    fn key(self) -> Key {
        self.object.key()
    }

    fn object_id(self) -> ObjectId {
        self.object.object_id().unwrap()
    }

    fn service_id(self, service: ServiceUuid) -> ServiceId {
        self.object.service_id(service).unwrap()
    }
}

#[derive(Debug)]
struct AnyObject<Key> {
    key: Key,
    services: HashMap<ServiceUuid, HashMap<ObjectUuid, ServiceCookie>>,
    created: HashMap<ObjectUuid, ObjectCookie>,
}

impl<Key> AnyObject<Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(key: Key, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        Self {
            key,
            services: services.into_iter().map(|s| (s, HashMap::new())).collect(),
            created: HashMap::new(),
        }
    }

    fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        if self.services.is_empty() {
            listener.add_filter(BusListenerFilter::any_object())?;
        } else {
            for service in self.services.keys() {
                listener.add_filter(BusListenerFilter::any_object_specific_service(*service))?;
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.created.clear();

        for cookies in self.services.values_mut() {
            cookies.clear();
        }
    }

    fn key(&self) -> Key {
        self.key
    }

    fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        self.created
            .get(&object)
            .map(|&cookie| ObjectId::new(object, cookie))
    }

    fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        self.object_id(object).map(|object_id| {
            ServiceId::new(
                object_id,
                service,
                *self
                    .services
                    .get(&service)
                    .expect("invalid UUID")
                    .get(&object)
                    .unwrap(),
            )
        })
    }

    fn iter(&self) -> AnyObjectIter<Key> {
        AnyObjectIter::new(self, self.created.iter())
    }

    fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match event {
            BusEvent::ObjectCreated(id) => self.object_created(id),
            BusEvent::ObjectDestroyed(id) => self.object_destroyed(id),
            BusEvent::ServiceCreated(id) => self.service_created(id),
            BusEvent::ServiceDestroyed(id) => self.service_destroyed(id),
        }
    }

    fn object_created(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if self.services.is_empty() {
            let dup = self.created.insert(id.uuid, id.cookie);
            debug_assert_eq!(dup, None);

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Created,
                id,
            ))
        } else {
            None
        }
    }

    fn object_destroyed(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if let Some(cookie) = self.created.remove(&id.uuid) {
            debug_assert_eq!(cookie, id.cookie);

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Destroyed,
                id,
            ))
        } else {
            None
        }
    }

    fn service_created(&mut self, id: ServiceId) -> Option<DiscovererEvent<Key>> {
        let service = self.services.get_mut(&id.uuid)?;
        let dup = service.insert(id.object_id.uuid, id.cookie);
        debug_assert_eq!(dup, None);

        if self
            .services
            .values()
            .all(|c| c.contains_key(&id.object_id.uuid))
        {
            let dup = self.created.insert(id.object_id.uuid, id.object_id.cookie);
            debug_assert_eq!(dup, None);

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Created,
                id.object_id,
            ))
        } else {
            None
        }
    }

    fn service_destroyed(&mut self, id: ServiceId) -> Option<DiscovererEvent<Key>> {
        let service = self.services.get_mut(&id.uuid)?;
        let cookie = service.remove(&id.object_id.uuid);
        debug_assert_eq!(cookie, Some(id.cookie));

        if let Some(cookie) = self.created.remove(&id.object_id.uuid) {
            debug_assert_eq!(cookie, id.object_id.cookie);

            Some(DiscovererEvent::new(
                self.key,
                DiscovererEventKind::Destroyed,
                id.object_id,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct AnyObjectIter<'a, Key> {
    object: &'a AnyObject<Key>,
    inner: hash_map::Iter<'a, ObjectUuid, ObjectCookie>,
}

impl<'a, Key> AnyObjectIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(
        object: &'a AnyObject<Key>,
        inner: hash_map::Iter<'a, ObjectUuid, ObjectCookie>,
    ) -> Self {
        Self { object, inner }
    }
}

impl<'a, Key> Iterator for AnyObjectIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = AnyObjectIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(uuid, cookie)| AnyObjectIterEntry::new(self.object, *uuid, *cookie))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug, Copy, Clone)]
struct AnyObjectIterEntry<'a, Key> {
    object: &'a AnyObject<Key>,
    object_id: ObjectId,
}

impl<'a, Key> AnyObjectIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(object: &'a AnyObject<Key>, uuid: ObjectUuid, cookie: ObjectCookie) -> Self {
        Self {
            object,
            object_id: ObjectId::new(uuid, cookie),
        }
    }

    fn key(self) -> Key {
        self.object.key()
    }

    fn object_id(self) -> ObjectId {
        self.object_id
    }

    fn service_id(self, service: ServiceUuid) -> ServiceId {
        self.object
            .service_id(self.object_id.uuid, service)
            .unwrap()
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

/// Event emitted by `Discoverer`s.
#[derive(Debug, Copy, Clone)]
pub struct DiscovererEvent<Key> {
    key: Key,
    kind: DiscovererEventKind,
    object: ObjectId,
}

impl<Key> DiscovererEvent<Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(key: Key, kind: DiscovererEventKind, object: ObjectId) -> Self {
        Self { key, kind, object }
    }

    /// Returns the key associated with this object.
    pub fn key(self) -> Key {
        self.key
    }

    /// Specifies whether the object was created or destroyed.
    pub fn kind(self) -> DiscovererEventKind {
        self.kind
    }

    /// Returns the object id that prompted this event.
    pub fn object_id(self) -> ObjectId {
        self.object
    }

    /// Returns a service id that is owned by this event's object.
    ///
    /// This function can only be called for those events that are emitted when an object is created
    /// ([`kind`](Self::kind) returns [`DiscovererEventKind::Created`]). It will panic otherwise.
    ///
    /// This function will also panic if `service` is not one of the UUIDs specified when
    /// [`object`](DiscovererBuilder::object) was called.
    pub fn service_id(self, discoverer: &Discoverer<Key>, service: ServiceUuid) -> ServiceId {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        discoverer
            .service_id(self.key, self.object.uuid, service)
            .unwrap()
    }
}

impl<Key> From<DiscovererIterEntry<'_, Key>> for DiscovererEvent<Key>
where
    Key: Copy + Eq + Hash,
{
    fn from(entry: DiscovererIterEntry<Key>) -> Self {
        Self::new(entry.key(), DiscovererEventKind::Created, entry.object_id())
    }
}
