use super::{
    DiscovererEvent, SpecificObjectWithServices, SpecificObjectWithServicesIter,
    SpecificObjectWithServicesIterEntry, SpecificObjectWithoutServices,
    SpecificObjectWithoutServicesIter, SpecificObjectWithoutServicesIterEntry,
};
use crate::Error;
use crate::bus_listener::BusListener;
use aldrin_core::{BusEvent, ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub(super) enum SpecificObject<Key> {
    WithServices(SpecificObjectWithServices<Key>),
    WithoutServices(SpecificObjectWithoutServices<Key>),
}

impl<Key> SpecificObject<Key>
where
    Key: Copy + Eq + Hash,
{
    pub(crate) fn new(
        key: Key,
        object: ObjectUuid,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        let services = services
            .into_iter()
            .map(|s| (s, None))
            .collect::<HashMap<_, _>>();

        if services.is_empty() {
            Self::WithoutServices(SpecificObjectWithoutServices::new(key, object))
        } else {
            Self::WithServices(SpecificObjectWithServices::new(key, object, services))
        }
    }

    pub(crate) fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        match self {
            Self::WithServices(inner) => inner.add_filter(listener),
            Self::WithoutServices(inner) => inner.add_filter(listener),
        }
    }

    pub(crate) fn reset(&mut self) {
        match self {
            Self::WithServices(inner) => inner.reset(),
            Self::WithoutServices(inner) => inner.reset(),
        }
    }

    pub(crate) fn key(&self) -> Key {
        match self {
            Self::WithServices(inner) => inner.key(),
            Self::WithoutServices(inner) => inner.key(),
        }
    }

    pub(crate) fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        match self {
            Self::WithServices(inner) => inner.object_id(object),
            Self::WithoutServices(inner) => inner.object_id(object),
        }
    }

    pub(crate) fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        match self {
            Self::WithServices(inner) => inner.service_id(object, service),
            Self::WithoutServices(_) => None,
        }
    }

    pub(crate) fn service_ids(
        &self,
        object: ObjectUuid,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Option<Vec<ServiceId>> {
        match self {
            Self::WithServices(inner) => inner.service_ids(object, services),
            Self::WithoutServices(inner) => Some(inner.service_ids(object, services)),
        }
    }

    pub(crate) fn service_ids_n<const N: usize>(
        &self,
        object: ObjectUuid,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        match self {
            Self::WithServices(inner) => inner.service_ids_n(object, services),
            Self::WithoutServices(inner) => Some(inner.service_ids_n(object, services)),
        }
    }

    pub(crate) fn contains(&self, object: ObjectUuid) -> bool {
        match self {
            Self::WithServices(inner) => inner.contains(object),
            Self::WithoutServices(inner) => inner.contains(object),
        }
    }

    pub(crate) fn contains_any(&self) -> bool {
        match self {
            Self::WithServices(inner) => inner.contains_any(),
            Self::WithoutServices(inner) => inner.contains_any(),
        }
    }

    pub(crate) fn iter(&self) -> SpecificObjectIter<'_, Key> {
        match self {
            Self::WithServices(inner) => SpecificObjectIter::WithServices(inner.iter()),
            Self::WithoutServices(inner) => SpecificObjectIter::WithoutServices(inner.iter()),
        }
    }

    pub(crate) fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match self {
            Self::WithServices(inner) => inner.handle_event(event),
            Self::WithoutServices(inner) => inner.handle_event(event),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum SpecificObjectIter<'a, Key> {
    WithServices(SpecificObjectWithServicesIter<'a, Key>),
    WithoutServices(SpecificObjectWithoutServicesIter<'a, Key>),
}

impl<'a, Key> Iterator for SpecificObjectIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = SpecificObjectIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::WithServices(inner) => inner.next().map(SpecificObjectIterEntry::WithServices),

            Self::WithoutServices(inner) => {
                inner.next().map(SpecificObjectIterEntry::WithoutServices)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::WithServices(inner) => inner.size_hint(),
            Self::WithoutServices(inner) => inner.size_hint(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) enum SpecificObjectIterEntry<'a, Key> {
    WithServices(SpecificObjectWithServicesIterEntry<'a, Key>),
    WithoutServices(SpecificObjectWithoutServicesIterEntry<'a, Key>),
}

impl<Key> SpecificObjectIterEntry<'_, Key>
where
    Key: Copy + Eq + Hash,
{
    pub(crate) fn key(self) -> Key {
        match self {
            Self::WithServices(inner) => inner.key(),
            Self::WithoutServices(inner) => inner.key(),
        }
    }

    pub(crate) fn object_id(self) -> ObjectId {
        match self {
            Self::WithServices(inner) => inner.object_id(),
            Self::WithoutServices(inner) => inner.object_id(),
        }
    }

    pub(crate) fn service_id(self, service: ServiceUuid) -> ServiceId {
        match self {
            Self::WithServices(inner) => inner.service_id(service),
            Self::WithoutServices(_) => panic!("invalid service UUID"),
        }
    }

    pub(crate) fn service_ids(
        self,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Vec<ServiceId> {
        match self {
            Self::WithServices(inner) => inner.service_ids(services),
            Self::WithoutServices(_) => panic!("invalid service UUID"),
        }
    }

    pub(crate) fn service_ids_n<const N: usize>(
        self,
        services: &[ServiceUuid; N],
    ) -> [ServiceId; N] {
        match self {
            Self::WithServices(inner) => inner.service_ids_n(services),
            Self::WithoutServices(_) => panic!("invalid service UUID"),
        }
    }
}
