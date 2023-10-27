use crate::ids::{ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use crate::message_deserializer::{MessageDeserializeError, MessageWithoutValueDeserializer};
use crate::message_serializer::MessageSerializer;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum BusEvent {
    ObjectCreated(ObjectId),
    ObjectDestroyed(ObjectId),
    ServiceCreated(ServiceId),
    ServiceDestroyed(ServiceId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum BusListenerScope {
    Current = 0,
    New = 1,
    All = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum BusListenerFilter {
    Object(Option<ObjectUuid>),
    Service(BusListenerServiceFilter),
}

impl BusListenerFilter {
    pub fn any_object() -> Self {
        Self::Object(None)
    }

    pub fn object(object: ObjectUuid) -> Self {
        Self::Object(Some(object))
    }

    pub fn service(filter: BusListenerServiceFilter) -> Self {
        Self::Service(filter)
    }

    pub fn any_object_any_service() -> Self {
        Self::service(BusListenerServiceFilter::any())
    }

    pub fn specific_object_any_service(object: ObjectUuid) -> Self {
        Self::service(BusListenerServiceFilter::with_object(object))
    }

    pub fn any_object_specific_service(service: ServiceUuid) -> Self {
        Self::service(BusListenerServiceFilter::with_service(service))
    }

    pub fn specific_object_and_service(object: ObjectUuid, service: ServiceUuid) -> Self {
        Self::service(BusListenerServiceFilter::with_object_and_service(
            object, service,
        ))
    }

    pub fn matches_object(self, object: ObjectId) -> bool {
        match self {
            Self::Object(None) => true,
            Self::Object(Some(filter)) => object.uuid == filter,
            Self::Service(_) => false,
        }
    }

    pub fn matches_service(self, service: ServiceId) -> bool {
        match self {
            Self::Object(_) => false,
            Self::Service(filter) => filter.matches(service),
        }
    }

    pub(crate) fn serialize_into_message(self, serializer: &mut MessageSerializer) {
        match self {
            Self::Object(None) => serializer.put_discriminant_u8(BusListenerFilterKind::AnyObject),

            Self::Object(Some(object)) => {
                serializer.put_discriminant_u8(BusListenerFilterKind::SpecificObject);
                serializer.put_uuid(object.0);
            }

            Self::Service(BusListenerServiceFilter {
                object: None,
                service: None,
            }) => serializer.put_discriminant_u8(BusListenerFilterKind::AnyObjectAnyService),

            Self::Service(BusListenerServiceFilter {
                object: Some(object),
                service: None,
            }) => {
                serializer.put_discriminant_u8(BusListenerFilterKind::SpecificObjectAnyService);
                serializer.put_uuid(object.0);
            }

            Self::Service(BusListenerServiceFilter {
                object: None,
                service: Some(service),
            }) => {
                serializer.put_discriminant_u8(BusListenerFilterKind::AnyObjectSpecificService);
                serializer.put_uuid(service.0);
            }

            Self::Service(BusListenerServiceFilter {
                object: Some(object),
                service: Some(service),
            }) => {
                serializer
                    .put_discriminant_u8(BusListenerFilterKind::SpecificObjectSpecificService);
                serializer.put_uuid(object.0);
                serializer.put_uuid(service.0);
            }
        }
    }

    pub(crate) fn deserialize_from_message(
        deserializer: &mut MessageWithoutValueDeserializer,
    ) -> Result<Self, MessageDeserializeError> {
        match deserializer.try_get_discriminant_u8()? {
            BusListenerFilterKind::AnyObject => Ok(Self::any_object()),

            BusListenerFilterKind::SpecificObject => {
                let object = deserializer.try_get_uuid().map(ObjectUuid)?;
                Ok(Self::object(object))
            }

            BusListenerFilterKind::AnyObjectAnyService => Ok(Self::any_object_any_service()),

            BusListenerFilterKind::SpecificObjectAnyService => {
                let object = deserializer.try_get_uuid().map(ObjectUuid)?;
                Ok(Self::specific_object_any_service(object))
            }

            BusListenerFilterKind::AnyObjectSpecificService => {
                let service = deserializer.try_get_uuid().map(ServiceUuid)?;
                Ok(Self::any_object_specific_service(service))
            }

            BusListenerFilterKind::SpecificObjectSpecificService => {
                let object = deserializer.try_get_uuid().map(ObjectUuid)?;
                let service = deserializer.try_get_uuid().map(ServiceUuid)?;
                Ok(Self::specific_object_and_service(object, service))
            }
        }
    }
}

impl From<ObjectUuid> for BusListenerFilter {
    fn from(object: ObjectUuid) -> Self {
        Self::Object(Some(object))
    }
}

impl From<Option<ObjectUuid>> for BusListenerFilter {
    fn from(object: Option<ObjectUuid>) -> Self {
        Self::Object(object)
    }
}

impl From<BusListenerServiceFilter> for BusListenerFilter {
    fn from(filter: BusListenerServiceFilter) -> Self {
        Self::Service(filter)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct BusListenerServiceFilter {
    pub object: Option<ObjectUuid>,
    pub service: Option<ServiceUuid>,
}

impl BusListenerServiceFilter {
    pub fn any() -> Self {
        Self {
            object: None,
            service: None,
        }
    }

    pub fn with_object(object: ObjectUuid) -> Self {
        Self {
            object: Some(object),
            service: None,
        }
    }

    pub fn with_service(service: ServiceUuid) -> Self {
        Self {
            object: None,
            service: Some(service),
        }
    }

    pub fn with_object_and_service(object: ObjectUuid, service: ServiceUuid) -> Self {
        Self {
            object: Some(object),
            service: Some(service),
        }
    }

    pub fn matches(self, id: ServiceId) -> bool {
        match (self.object, self.service) {
            (None, None) => true,
            (Some(object), None) => id.object_id.uuid == object,
            (None, Some(service)) => id.uuid == service,
            (Some(object), Some(service)) => (id.object_id.uuid == object) && (id.uuid == service),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum BusListenerFilterKind {
    AnyObject = 0,
    SpecificObject = 1,
    AnyObjectAnyService = 2,
    SpecificObjectAnyService = 3,
    AnyObjectSpecificService = 4,
    SpecificObjectSpecificService = 5,
}
