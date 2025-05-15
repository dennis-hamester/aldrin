use super::{Discoverer, DiscovererIterEntry};
use aldrin_core::{ObjectId, ServiceId, ServiceUuid};
use std::hash::Hash;

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
    pub(super) fn new(key: Key, kind: DiscovererEventKind, object: ObjectId) -> Self {
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

    /// Checks if this is a [`DiscovererEventKind::Created`] event.
    pub fn is_created(self) -> bool {
        self.kind == DiscovererEventKind::Created
    }

    /// Checks if this is a [`DiscovererEventKind::Destroyed`] event.
    pub fn is_destroyed(self) -> bool {
        self.kind == DiscovererEventKind::Destroyed
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
    /// [`add`](super::DiscovererBuilder::add) was called.
    pub fn service_id(
        self,
        discoverer: &Discoverer<Key>,
        service: impl Into<ServiceUuid>,
    ) -> ServiceId {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        discoverer
            .service_id(self.key, self.object.uuid, service)
            .unwrap()
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids<S>(self, discoverer: &Discoverer<Key>, services: S) -> Vec<ServiceId>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        discoverer
            .service_ids(self.key, self.object.uuid, services)
            .unwrap()
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids_n<const N: usize>(
        self,
        discoverer: &Discoverer<Key>,
        services: &[ServiceUuid; N],
    ) -> [ServiceId; N] {
        assert_eq!(self.kind, DiscovererEventKind::Created);

        discoverer
            .service_ids_n(self.key, self.object.uuid, services)
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

/// Specifies whether an object was created or destroyed.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiscovererEventKind {
    /// An object was created.
    Created,

    /// An object was destroyed.
    Destroyed,
}
