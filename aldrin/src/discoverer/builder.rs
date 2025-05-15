use super::{AnyObject, Discoverer, DiscovererEntry, SpecificObject};
use crate::{Error, Handle};
use aldrin_core::{ObjectUuid, ServiceUuid};
use std::collections::HashMap;
use std::hash::Hash;

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

    /// Adds an object to the discoverer.
    ///
    /// The `key` is an arbitrary value that can later be queried again on
    /// [`DiscovererEvent`s](super::DiscovererEvent). It can be used to distinguish events when more
    /// than one object has been added.
    ///
    /// When specifying an [`ObjectUuid`], the discoverer will match only on that UUID. Otherwise,
    /// the discoverer will emit events for every object that matches the set of services.
    pub fn add(
        mut self,
        key: Key,
        object: Option<ObjectUuid>,
        services: impl IntoIterator<Item = ServiceUuid>,
    ) -> Self {
        let entry = match object {
            Some(object) => SpecificObject::new(key, object, services).into(),
            None => AnyObject::new(key, services).into(),
        };

        self.entries.insert(key, entry);
        self
    }

    /// Registers interest in a specific object implementing a set of services.
    ///
    /// This is a shorthand for calling `add(key, Some(object), services)`.
    pub fn object_with_services<S>(
        self,
        key: Key,
        object: impl Into<ObjectUuid>,
        services: S,
    ) -> Self
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.add(
            key,
            Some(object.into()),
            services.into_iter().map(Into::into),
        )
    }

    /// Registers interest in a specific object without any services.
    ///
    /// This is a shorthand for calling `add(key, Some(object), [])`.
    pub fn bare_object(self, key: Key, object: impl Into<ObjectUuid>) -> Self {
        self.add(key, Some(object.into()), None)
    }

    /// Registers interest in a any object implementing a set of services.
    ///
    /// This is a shorthand for calling `add(key, None, services)`.
    pub fn any_object_with_services<S>(self, key: Key, services: S) -> Self
    where
        S: IntoIterator,
        S::Item: Into<ServiceUuid>,
    {
        self.add(key, None, services.into_iter().map(Into::into))
    }
}
