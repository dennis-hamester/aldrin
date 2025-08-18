use super::{DiscovererEvent, DiscovererEventKind};
use crate::bus_listener::BusListener;
use crate::Error;
use aldrin_core::{
    BusEvent, BusListenerFilter, ObjectCookie, ObjectId, ObjectUuid, ServiceId, ServiceUuid,
};
use std::hash::Hash;
use std::option;

#[derive(Debug)]
pub(super) struct SpecificObjectWithoutServices<Key> {
    key: Key,
    object: ObjectUuid,
    cookie: Option<ObjectCookie>,
}

impl<Key> SpecificObjectWithoutServices<Key>
where
    Key: Copy + Eq + Hash,
{
    pub(crate) fn new(key: Key, object: ObjectUuid) -> Self {
        Self {
            key,
            object,
            cookie: None,
        }
    }

    pub(crate) fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        listener.add_filter(BusListenerFilter::object(self.object))?;
        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        self.cookie = None;
    }

    pub(crate) fn key(&self) -> Key {
        self.key
    }

    fn object_id_unchecked(&self) -> ObjectId {
        ObjectId::new(self.object, self.cookie.unwrap())
    }

    pub(crate) fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        assert_eq!(object, self.object);
        self.cookie.map(|cookie| ObjectId::new(self.object, cookie))
    }

    pub(crate) fn service_ids(
        &self,
        object: ObjectUuid,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Option<Vec<ServiceId>> {
        assert_eq!(object, self.object);

        if services.into_iter().next().is_none() {
            Some(Vec::new())
        } else {
            panic!("invalid service UUID")
        }
    }

    pub(crate) fn service_ids_n<const N: usize>(
        &self,
        object: ObjectUuid,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        assert_eq!(object, self.object);

        if services.is_empty() {
            Some(super::fill_service_id_array(services, |_| unreachable!()))
        } else {
            panic!("invalid service UUID")
        }
    }

    pub(crate) fn contains(&self, object: ObjectUuid) -> bool {
        self.cookie.is_some() && (object == self.object)
    }

    pub(crate) fn contains_any(&self) -> bool {
        self.cookie.is_some()
    }

    pub(crate) fn iter(&self) -> SpecificObjectWithoutServicesIter<'_, Key> {
        if self.cookie.is_some() {
            SpecificObjectWithoutServicesIter::new(Some(
                SpecificObjectWithoutServicesIterEntry::new(self),
            ))
        } else {
            SpecificObjectWithoutServicesIter::new(None)
        }
    }

    pub(crate) fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match event {
            BusEvent::ObjectCreated(id) => self.object_created(id),
            BusEvent::ObjectDestroyed(id) => self.object_destroyed(id),
            BusEvent::ServiceCreated(_) | BusEvent::ServiceDestroyed(_) => None,
        }
    }

    fn object_created(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if id.uuid != self.object {
            return None;
        }

        debug_assert!(self.cookie.is_none());
        self.cookie = Some(id.cookie);

        Some(DiscovererEvent::new(
            self.key,
            DiscovererEventKind::Created,
            id,
        ))
    }

    fn object_destroyed(&mut self, id: ObjectId) -> Option<DiscovererEvent<Key>> {
        if id.uuid != self.object {
            return None;
        }

        debug_assert_eq!(self.cookie, Some(id.cookie));
        self.cookie = None;

        Some(DiscovererEvent::new(
            self.key,
            DiscovererEventKind::Destroyed,
            id,
        ))
    }
}

#[derive(Debug, Clone)]
pub(super) struct SpecificObjectWithoutServicesIter<'a, Key> {
    next: option::IntoIter<SpecificObjectWithoutServicesIterEntry<'a, Key>>,
}

impl<'a, Key> SpecificObjectWithoutServicesIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(next: Option<SpecificObjectWithoutServicesIterEntry<'a, Key>>) -> Self {
        Self {
            next: next.into_iter(),
        }
    }
}

impl<'a, Key> Iterator for SpecificObjectWithoutServicesIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = SpecificObjectWithoutServicesIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.next.size_hint()
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct SpecificObjectWithoutServicesIterEntry<'a, Key> {
    object: &'a SpecificObjectWithoutServices<Key>,
}

impl<'a, Key> SpecificObjectWithoutServicesIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(object: &'a SpecificObjectWithoutServices<Key>) -> Self {
        Self { object }
    }

    pub(crate) fn key(self) -> Key {
        self.object.key()
    }

    pub(crate) fn object_id(self) -> ObjectId {
        self.object.object_id_unchecked()
    }
}
