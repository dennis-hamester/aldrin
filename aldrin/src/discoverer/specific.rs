use super::{DiscovererEvent, DiscovererEventKind};
use crate::bus_listener::BusListener;
use crate::Error;
use aldrin_core::{
    BusEvent, BusListenerFilter, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::option;

#[derive(Debug)]
pub(super) struct SpecificObject<Key> {
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
    pub fn new(
        key: Key,
        object: ObjectUuid,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        Self {
            key,
            object,
            cookie: None,
            services: services.into_iter().map(|s| (s, None)).collect(),
            created: false,
        }
    }

    pub fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        listener.add_filter(BusListenerFilter::object(self.object))?;

        for service in self.services.keys() {
            listener.add_filter(BusListenerFilter::specific_object_and_service(
                self.object,
                *service,
            ))?;
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.cookie = None;
        self.created = false;

        for cookie in self.services.values_mut() {
            *cookie = None;
        }
    }

    pub fn key(&self) -> Key {
        self.key
    }

    fn object_id_unchecked(&self) -> ObjectId {
        ObjectId::new(self.object, self.cookie.unwrap())
    }

    pub fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        assert_eq!(object, self.object);
        self.created.then(|| self.object_id_unchecked())
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

    pub fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
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

    pub fn service_ids(
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

    pub fn service_ids_n<const N: usize>(
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

    pub fn contains(&self, object: ObjectUuid) -> bool {
        self.created && (object == self.object)
    }

    pub fn contains_any(&self) -> bool {
        self.created
    }

    pub fn iter(&self) -> SpecificObjectIter<'_, Key> {
        if self.created {
            SpecificObjectIter::new(Some(SpecificObjectIterEntry::new(self)))
        } else {
            SpecificObjectIter::new(None)
        }
    }

    pub fn handle_event(&mut self, event: BusEvent) -> Option<DiscovererEvent<Key>> {
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
pub(super) struct SpecificObjectIter<'a, Key> {
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
pub(super) struct SpecificObjectIterEntry<'a, Key> {
    object: &'a SpecificObject<Key>,
}

impl<'a, Key> SpecificObjectIterEntry<'a, Key>
where
    Key: Copy + Eq + Hash,
{
    fn new(object: &'a SpecificObject<Key>) -> Self {
        Self { object }
    }

    pub fn key(self) -> Key {
        self.object.key()
    }

    pub fn object_id(self) -> ObjectId {
        self.object.object_id_unchecked()
    }

    pub fn service_id(self, service: ServiceUuid) -> ServiceId {
        self.object.service_id_unchecked(service)
    }

    pub fn service_ids(self, services: impl IntoIterator<Item = ServiceUuid>) -> Vec<ServiceId> {
        self.object.service_ids_unchecked(services)
    }

    pub fn service_ids_n<const N: usize>(self, services: &[ServiceUuid; N]) -> [ServiceId; N] {
        self.object.service_ids_n_unchecked(services)
    }
}
