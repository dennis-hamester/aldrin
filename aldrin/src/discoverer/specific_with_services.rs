use super::{DiscovererEvent, DiscovererEventKind};
use crate::Error;
use crate::bus_listener::BusListener;
use aldrin_core::{
    BusEvent, BusListenerFilter, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::option;

#[derive(Debug)]
pub(super) struct SpecificObjectWithServices<Key> {
    key: Key,
    object: ObjectUuid,
    cookie: Option<ObjectCookie>,
    services: HashMap<ServiceUuid, Option<ServiceCookie>>,
}

impl<Key> SpecificObjectWithServices<Key>
where
    Key: Copy + Eq + Hash,
{
    pub(crate) fn new(
        key: Key,
        object: ObjectUuid,
        services: HashMap<ServiceUuid, Option<ServiceCookie>>,
    ) -> Self {
        debug_assert!(!services.is_empty());

        Self {
            key,
            object,
            cookie: None,
            services,
        }
    }

    pub(crate) fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        for service in self.services.keys() {
            listener.add_filter(BusListenerFilter::specific_object_and_service(
                self.object,
                *service,
            ))?;
        }

        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        self.cookie = None;

        for cookie in self.services.values_mut() {
            *cookie = None;
        }
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

    fn service_cookie_unchecked(&self, service: ServiceUuid) -> ServiceCookie {
        self.services
            .get(&service)
            .expect("valid service UUID")
            .unwrap()
    }

    fn service_id_unchecked(&self, service: ServiceUuid) -> ServiceId {
        let object_id = self.object_id_unchecked();
        let service_cookie = self.service_cookie_unchecked(service);
        ServiceId::new(object_id, service, service_cookie)
    }

    pub(crate) fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        self.object_id(object).map(|object_id| {
            ServiceId::new(object_id, service, self.service_cookie_unchecked(service))
        })
    }

    fn service_ids_unchecked(
        &self,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Vec<ServiceId> {
        let object_id = self.object_id_unchecked();

        services
            .into_iter()
            .map(|service_uuid| {
                let service_cookie = self.service_cookie_unchecked(service_uuid);
                ServiceId::new(object_id, service_uuid, service_cookie)
            })
            .collect()
    }

    pub(crate) fn service_ids(
        &self,
        object: ObjectUuid,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Option<Vec<ServiceId>> {
        self.object_id(object).map(|object_id| {
            services
                .into_iter()
                .map(|service_uuid| {
                    let service_cookie = self.service_cookie_unchecked(service_uuid);
                    ServiceId::new(object_id, service_uuid, service_cookie)
                })
                .collect()
        })
    }

    fn service_ids_n_unchecked<const N: usize>(
        &self,
        services: &[ServiceUuid; N],
    ) -> [ServiceId; N] {
        let object_id = self.object_id_unchecked();

        super::fill_service_id_array(services, |service_uuid| {
            let service_cookie = self.service_cookie_unchecked(service_uuid);
            ServiceId::new(object_id, service_uuid, service_cookie)
        })
    }

    pub(crate) fn service_ids_n<const N: usize>(
        &self,
        object: ObjectUuid,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        self.object_id(object).map(|object_id| {
            super::fill_service_id_array(services, |service_uuid| {
                let service_cookie = self.service_cookie_unchecked(service_uuid);
                ServiceId::new(object_id, service_uuid, service_cookie)
            })
        })
    }

    pub(crate) fn contains(&self, object: ObjectUuid) -> bool {
        self.cookie.is_some() && (object == self.object)
    }

    pub(crate) fn contains_any(&self) -> bool {
        self.cookie.is_some()
    }

    pub(crate) fn iter(&self) -> SpecificObjectWithServicesIter<'_, Key> {
        if self.cookie.is_some() {
            SpecificObjectWithServicesIter::new(Some(SpecificObjectWithServicesIterEntry::new(
                self,
            )))
        } else {
            SpecificObjectWithServicesIter::new(None)
        }
    }

    pub(crate) fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
        match event {
            BusEvent::ServiceCreated(id) => self.service_created(id),
            BusEvent::ServiceDestroyed(id) => self.service_destroyed(id),
            BusEvent::ObjectCreated(_) | BusEvent::ObjectDestroyed(_) => None,
        }
    }

    fn service_created(&mut self, id: ServiceId) -> Option<DiscovererEvent<Key>> {
        if id.object_id.uuid != self.object {
            return None;
        }

        // self.cookie.get_or_insert(id.object_id.cookie);
        // debug_assert_eq!(self.cookie, Some(id.object_id.cookie));

        let service = self.services.get_mut(&id.uuid)?;
        debug_assert!(service.is_none());
        *service = Some(id.cookie);

        if self.services.values().all(Option::is_some) {
            self.cookie = Some(id.object_id.cookie);

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

        let service = self.services.get_mut(&id.uuid)?;
        debug_assert_eq!(*service, Some(id.cookie));
        *service = None;

        if self.cookie.is_some() {
            self.cookie = None;

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
pub(super) struct SpecificObjectWithServicesIter<'a, Key> {
    next: option::IntoIter<SpecificObjectWithServicesIterEntry<'a, Key>>,
}

impl<'a, Key> SpecificObjectWithServicesIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(next: Option<SpecificObjectWithServicesIterEntry<'a, Key>>) -> Self {
        Self {
            next: next.into_iter(),
        }
    }
}

impl<'a, Key> Iterator for SpecificObjectWithServicesIter<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    type Item = SpecificObjectWithServicesIterEntry<'a, Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.next.size_hint()
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct SpecificObjectWithServicesIterEntry<'a, Key> {
    object: &'a SpecificObjectWithServices<Key>,
}

impl<'a, Key> SpecificObjectWithServicesIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(object: &'a SpecificObjectWithServices<Key>) -> Self {
        Self { object }
    }

    pub(crate) fn key(self) -> Key {
        self.object.key()
    }

    pub(crate) fn object_id(self) -> ObjectId {
        self.object.object_id_unchecked()
    }

    pub(crate) fn service_id(self, service: ServiceUuid) -> ServiceId {
        self.object.service_id_unchecked(service)
    }

    pub(crate) fn service_ids(
        self,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Vec<ServiceId> {
        self.object.service_ids_unchecked(services)
    }

    pub(crate) fn service_ids_n<const N: usize>(
        self,
        services: &[ServiceUuid; N],
    ) -> [ServiceId; N] {
        self.object.service_ids_n_unchecked(services)
    }
}
