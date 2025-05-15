//! Contains the [`Discoverer`] and its related types.

mod any;
mod builder;
mod entry;
mod event;
mod specific;
#[cfg(test)]
mod test;

use crate::bus_listener::BusListener;
use crate::{Error, Handle};
use aldrin_core::{BusListenerScope, ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use any::{AnyObject, AnyObjectIter, AnyObjectIterEntry};
use futures_core::stream::{FusedStream, Stream};
use specific::{SpecificObject, SpecificObjectIter, SpecificObjectIterEntry};
use std::collections::hash_map::{self, HashMap};
use std::collections::VecDeque;
use std::future;
use std::hash::Hash;
use std::iter::{FlatMap, FusedIterator};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use builder::DiscovererBuilder;
pub use entry::{DiscovererEntry, DiscovererEntryIter, DiscovererIterEntry};
pub use event::{DiscovererEvent, DiscovererEventKind};

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
/// # use aldrin::Discoverer;
/// # use aldrin::discoverer::DiscovererBuilder;
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
///     .object_with_services(1, OBJECT_UUID, [SERVICE_UUID_1, SERVICE_UUID_2])
///     .any_object_with_services(2, [SERVICE_UUID_1])
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
    pub fn object_id(&self, key: Key, object: impl Into<ObjectUuid>) -> Option<ObjectId> {
        self.entry(key).object_id(object)
    }

    /// Queries a specific service id.
    pub fn service_id(
        &self,
        key: Key,
        object: impl Into<ObjectUuid>,
        service: impl Into<ServiceUuid>,
    ) -> Option<ServiceId> {
        self.entry(key).service_id(object, service)
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids<S>(
        &self,
        key: Key,
        object: impl Into<ObjectUuid>,
        services: S,
    ) -> Option<Vec<ServiceId>>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.entry(key).service_ids(object, services)
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids_n<const N: usize>(
        &self,
        key: Key,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        self.entry(key).service_ids_n(object, services)
    }

    /// Returns an entry of the `Discoverer`.
    pub fn entry(&self, key: Key) -> &DiscovererEntry<Key> {
        self.entries.get(&key).expect("valid key")
    }

    /// Returns an iterator over all found objects.
    pub fn iter(&self) -> DiscovererIter<Key> {
        DiscovererIter::new(self.entries.values().flat_map(DiscovererEntry::iter))
    }

    /// Returns an iterator over all found objects corresponding to a specific key.
    pub fn entry_iter(&self, key: Key) -> DiscovererEntryIter<Key> {
        self.entry(key).iter()
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

impl<Key> FusedIterator for DiscovererIter<'_, Key> where Key: Copy + Eq + Hash {}

fn fill_service_id_array<const N: usize>(
    service_uuids: &[ServiceUuid; N],
    get_id: impl Fn(ServiceUuid) -> ServiceId,
) -> [ServiceId; N] {
    // SAFETY: This creates an array of MaybeUninit, which doesn't require initialization.
    let mut ids: [MaybeUninit<ServiceId>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    for (&uuid, id) in service_uuids.iter().zip(&mut ids) {
        id.write(get_id(uuid));
    }

    // SAFETY: All N elements have been initialized in the loop above.
    //
    // In some future version of Rust, all this can be simplified; see:
    // https://github.com/rust-lang/rust/issues/96097
    // https://github.com/rust-lang/rust/issues/61956
    unsafe {
        (*(&MaybeUninit::new(ids) as *const _ as *const MaybeUninit<[ServiceId; N]>))
            .assume_init_read()
    }
}
