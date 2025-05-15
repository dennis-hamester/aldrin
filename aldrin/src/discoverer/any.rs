use super::{DiscovererEvent, DiscovererEventKind};
use crate::bus_listener::BusListener;
use crate::Error;
use aldrin_core::{
    BusEvent, BusListenerFilter, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId,
    ServiceUuid,
};
use std::collections::hash_map::{self, HashMap};
use std::hash::Hash;

#[derive(Debug)]
pub(super) struct AnyObject<Key> {
    key: Key,
    services: HashMap<ServiceUuid, HashMap<ObjectUuid, ServiceCookie>>,
    created: HashMap<ObjectUuid, ObjectCookie>,
}

impl<Key> AnyObject<Key>
where
    Key: Copy + Eq + Hash,
{
    pub fn new(key: Key, services: impl IntoIterator<Item = ServiceUuid>) -> Self {
        Self {
            key,
            services: services.into_iter().map(|s| (s, HashMap::new())).collect(),
            created: HashMap::new(),
        }
    }

    pub fn add_filter(&self, listener: &mut BusListener) -> Result<(), Error> {
        if self.services.is_empty() {
            listener.add_filter(BusListenerFilter::any_object())?;
        } else {
            for service in self.services.keys() {
                listener.add_filter(BusListenerFilter::any_object_specific_service(*service))?;
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.created.clear();

        for cookies in self.services.values_mut() {
            cookies.clear();
        }
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn object_id(&self, object: ObjectUuid) -> Option<ObjectId> {
        self.created
            .get(&object)
            .map(|&cookie| ObjectId::new(object, cookie))
    }

    fn service_cookie_unchecked(&self, object: ObjectUuid, service: ServiceUuid) -> ServiceCookie {
        *self
            .services
            .get(&service)
            .expect("valid service UUID")
            .get(&object)
            .unwrap()
    }

    pub fn service_id(&self, object: ObjectUuid, service: ServiceUuid) -> Option<ServiceId> {
        self.object_id(object).map(|object_id| {
            ServiceId::new(
                object_id,
                service,
                self.service_cookie_unchecked(object, service),
            )
        })
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
                    let service_cookie = self.service_cookie_unchecked(object, service_uuid);
                    ServiceId::new(object_id, service_uuid, service_cookie)
                })
                .collect()
        })
    }

    pub fn service_ids_n<const N: usize>(
        &self,
        object: ObjectUuid,
        services: &[ServiceUuid; N],
    ) -> Option<[ServiceId; N]> {
        self.object_id(object).map(|object_id| {
            super::fill_service_id_array(services, |service_uuid| {
                let service_cookie = self.service_cookie_unchecked(object, service_uuid);
                ServiceId::new(object_id, service_uuid, service_cookie)
            })
        })
    }

    pub fn iter(&self) -> AnyObjectIter<Key> {
        AnyObjectIter::new(self, self.created.iter())
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
pub(super) struct AnyObjectIter<'a, Key> {
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
pub(super) struct AnyObjectIterEntry<'a, Key> {
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

    pub fn key(self) -> Key {
        self.object.key()
    }

    pub fn object_id(self) -> ObjectId {
        self.object_id
    }

    pub fn service_id(self, service: ServiceUuid) -> ServiceId {
        self.object
            .service_id(self.object_id.uuid, service)
            .unwrap()
    }

    pub fn service_ids(self, services: impl IntoIterator<Item = ServiceUuid>) -> Vec<ServiceId> {
        self.object
            .service_ids(self.object_id.uuid, services)
            .unwrap()
    }

    pub fn service_ids_n<const N: usize>(self, services: &[ServiceUuid; N]) -> [ServiceId; N] {
        self.object
            .service_ids_n(self.object_id.uuid, services)
            .unwrap()
    }
}
