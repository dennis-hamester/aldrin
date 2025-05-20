use super::{
    AnyObject, AnyObjectIter, AnyObjectIterEntry, DiscovererEvent, SpecificObject,
    SpecificObjectIter, SpecificObjectIterEntry,
};
use crate::bus_listener::BusListener;
use crate::Error;
use aldrin_core::{BusEvent, ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use std::hash::Hash;
use std::iter::FusedIterator;

/// Entry of a `Discoverer`.
#[derive(Debug)]
pub struct DiscovererEntry<Key> {
    inner: EntryInner<Key>,
}

impl<Key> DiscovererEntry<Key>
where
    Key: Copy + Eq + Hash,
{
    pub(super) fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.add_filter(listener),
            EntryInner::Any(ref any) => any.add_filter(listener),
        }
    }

    pub(super) fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match self.inner {
            EntryInner::Specific(ref mut specific) => specific.handle_event(event),
            EntryInner::Any(ref mut any) => any.handle_event(event),
        }
    }

    pub(super) fn reset(&mut self) {
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
    pub fn object_id(&self, object: impl Into<ObjectUuid>) -> Option<ObjectId> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.object_id(object.into()),
            EntryInner::Any(ref any) => any.object_id(object.into()),
        }
    }

    /// Queries a `ServiceId` of an existing object.
    pub fn service_id(
        &self,
        object: impl Into<ObjectUuid>,
        service: impl Into<ServiceUuid>,
    ) -> Option<ServiceId> {
        match self.inner {
            EntryInner::Specific(ref specific) => {
                specific.service_id(object.into(), service.into())
            }

            EntryInner::Any(ref any) => any.service_id(object.into(), service.into()),
        }
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids<S>(
        &self,
        object: impl Into<ObjectUuid>,
        services: S,
    ) -> Option<Vec<ServiceId>>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        match self.inner {
            EntryInner::Specific(ref specific) => {
                specific.service_ids(object.into(), services.into_iter().map(Into::into))
            }

            EntryInner::Any(ref any) => {
                any.service_ids(object.into(), services.into_iter().map(Into::into))
            }
        }
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids_n<const N: usize>(
        &self,
        object: impl Into<ObjectUuid>,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.service_ids_n(object.into(), services),
            EntryInner::Any(ref any) => any.service_ids_n(object.into(), services),
        }
    }

    /// Checks if there is a specific object associated with this entry.
    pub fn contains(&self, object: impl Into<ObjectUuid>) -> bool {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.contains(object.into()),
            EntryInner::Any(ref any) => any.contains(object.into()),
        }
    }

    /// Checks if there are any known objects associated with this entry.
    pub fn contains_any(&self) -> bool {
        match self.inner {
            EntryInner::Specific(ref specific) => specific.contains_any(),
            EntryInner::Any(ref any) => any.contains_any(),
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

impl<Key> FusedIterator for DiscovererEntryIter<'_, Key> where Key: Copy + Eq + Hash {}

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

/// Item type when iterating over a [`Discoverer`](super::Discoverer) or [`DiscovererEntry`].
#[derive(Debug, Copy, Clone)]
pub struct DiscovererIterEntry<'a, Key> {
    inner: IterEntryInner<'a, Key>,
}

impl<Key> DiscovererIterEntry<'_, Key>
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
    pub fn service_id(self, service: impl Into<ServiceUuid>) -> ServiceId {
        match self.inner {
            IterEntryInner::Specific(specific) => specific.service_id(service.into()),
            IterEntryInner::Any(any) => any.service_id(service.into()),
        }
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids<S>(self, services: S) -> Vec<ServiceId>
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        match self.inner {
            IterEntryInner::Specific(specific) => {
                specific.service_ids(services.into_iter().map(Into::into))
            }

            IterEntryInner::Any(any) => any.service_ids(services.into_iter().map(Into::into)),
        }
    }

    /// Queries multiple service ids.
    ///
    /// The ids are returned in the same order as specified in `services`.
    pub fn service_ids_n<const N: usize>(self, services: &[ServiceUuid; N]) -> [ServiceId; N] {
        match self.inner {
            IterEntryInner::Specific(specific) => specific.service_ids_n(services),
            IterEntryInner::Any(any) => any.service_ids_n(services),
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
