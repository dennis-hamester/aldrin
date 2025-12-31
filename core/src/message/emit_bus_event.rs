use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer, OptionKind,
};
use crate::{
    BusEvent, BusListenerCookie, ObjectId, SerializedValue, SerializedValueSlice, ServiceId,
};
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct EmitBusEvent {
    pub cookie: Option<BusListenerCookie>,
    pub event: BusEvent,
}

impl MessageOps for EmitBusEvent {
    fn kind(&self) -> MessageKind {
        MessageKind::EmitBusEvent
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::EmitBusEvent);

        match self.cookie {
            Some(cookie) => {
                serializer.put_discriminant_u8(OptionKind::Some);
                serializer.put_uuid(cookie.0);
            }

            None => serializer.put_discriminant_u8(OptionKind::None),
        }

        match self.event {
            BusEvent::ObjectCreated(object) => {
                serializer.put_discriminant_u8(BusEventKind::ObjectCreated);
                serializer.put_uuid(object.uuid.0);
                serializer.put_uuid(object.cookie.0);
            }

            BusEvent::ObjectDestroyed(object) => {
                serializer.put_discriminant_u8(BusEventKind::ObjectDestroyed);
                serializer.put_uuid(object.uuid.0);
                serializer.put_uuid(object.cookie.0);
            }

            BusEvent::ServiceCreated(service) => {
                serializer.put_discriminant_u8(BusEventKind::ServiceCreated);
                serializer.put_uuid(service.object_id.uuid.0);
                serializer.put_uuid(service.object_id.cookie.0);
                serializer.put_uuid(service.uuid.0);
                serializer.put_uuid(service.cookie.0);
            }

            BusEvent::ServiceDestroyed(service) => {
                serializer.put_discriminant_u8(BusEventKind::ServiceDestroyed);
                serializer.put_uuid(service.object_id.uuid.0);
                serializer.put_uuid(service.object_id.cookie.0);
                serializer.put_uuid(service.uuid.0);
                serializer.put_uuid(service.cookie.0);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::EmitBusEvent)?;

        let cookie = match deserializer.try_get_discriminant_u8()? {
            OptionKind::Some => deserializer
                .try_get_uuid()
                .map(BusListenerCookie::from)
                .map(Some)?,

            OptionKind::None => None,
        };

        let event = match deserializer.try_get_discriminant_u8()? {
            BusEventKind::ObjectCreated => {
                let object_uuid = deserializer.try_get_uuid()?.into();
                let object_cookie = deserializer.try_get_uuid()?.into();

                BusEvent::ObjectCreated(ObjectId::new(object_uuid, object_cookie))
            }

            BusEventKind::ObjectDestroyed => {
                let object_uuid = deserializer.try_get_uuid()?.into();
                let object_cookie = deserializer.try_get_uuid()?.into();

                BusEvent::ObjectDestroyed(ObjectId::new(object_uuid, object_cookie))
            }

            BusEventKind::ServiceCreated => {
                let object_uuid = deserializer.try_get_uuid()?.into();
                let object_cookie = deserializer.try_get_uuid()?.into();
                let service_uuid = deserializer.try_get_uuid()?.into();
                let service_cookie = deserializer.try_get_uuid()?.into();

                BusEvent::ServiceCreated(ServiceId::new(
                    ObjectId::new(object_uuid, object_cookie),
                    service_uuid,
                    service_cookie,
                ))
            }

            BusEventKind::ServiceDestroyed => {
                let object_uuid = deserializer.try_get_uuid()?.into();
                let object_cookie = deserializer.try_get_uuid()?.into();
                let service_uuid = deserializer.try_get_uuid()?.into();
                let service_cookie = deserializer.try_get_uuid()?.into();

                BusEvent::ServiceDestroyed(ServiceId::new(
                    ObjectId::new(object_uuid, object_cookie),
                    service_uuid,
                    service_cookie,
                ))
            }
        };

        deserializer.finish()?;

        Ok(Self { cookie, event })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for EmitBusEvent {}

impl From<EmitBusEvent> for Message {
    fn from(msg: EmitBusEvent) -> Self {
        Self::EmitBusEvent(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::EmitBusEvent;
    use crate::{BusEvent, BusListenerCookie, ObjectId, ServiceId};
    use uuid::uuid;

    #[test]
    fn object_created_without_cookie() {
        let serialized = [
            39, 0, 0, 0, 44, 0, 0, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc, 0x5e,
            0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49, 0x30,
            0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7,
        ];

        let msg = EmitBusEvent {
            cookie: None,
            event: BusEvent::ObjectCreated(ObjectId::new(
                uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn object_destroyed_without_cookie() {
        let serialized = [
            39, 0, 0, 0, 44, 0, 1, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc, 0x5e,
            0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49, 0x30,
            0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7,
        ];

        let msg = EmitBusEvent {
            cookie: None,
            event: BusEvent::ObjectDestroyed(ObjectId::new(
                uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn service_created_without_cookie() {
        let serialized = [
            71, 0, 0, 0, 44, 0, 2, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc, 0x5e,
            0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49, 0x30,
            0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7, 0xaf, 0xfc, 0xb7, 0x9b, 0xe9, 0x9c,
            0x47, 0x84, 0x95, 0xe0, 0x0d, 0xc9, 0x9f, 0xcb, 0xa8, 0xff, 0x37, 0x22, 0x74, 0x6e,
            0x76, 0xa8, 0x41, 0x31, 0x98, 0x32, 0x42, 0xc4, 0x78, 0x3f, 0x89, 0x97,
        ];

        let msg = EmitBusEvent {
            cookie: None,
            event: BusEvent::ServiceCreated(ServiceId::new(
                ObjectId::new(
                    uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                    uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
                ),
                uuid!("affcb79b-e99c-4784-95e0-0dc99fcba8ff").into(),
                uuid!("3722746e-76a8-4131-9832-42c4783f8997").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn service_destroyed_without_cookie() {
        let serialized = [
            71, 0, 0, 0, 44, 0, 3, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc, 0x5e,
            0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49, 0x30,
            0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7, 0xaf, 0xfc, 0xb7, 0x9b, 0xe9, 0x9c,
            0x47, 0x84, 0x95, 0xe0, 0x0d, 0xc9, 0x9f, 0xcb, 0xa8, 0xff, 0x37, 0x22, 0x74, 0x6e,
            0x76, 0xa8, 0x41, 0x31, 0x98, 0x32, 0x42, 0xc4, 0x78, 0x3f, 0x89, 0x97,
        ];

        let msg = EmitBusEvent {
            cookie: None,
            event: BusEvent::ServiceDestroyed(ServiceId::new(
                ObjectId::new(
                    uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                    uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
                ),
                uuid!("affcb79b-e99c-4784-95e0-0dc99fcba8ff").into(),
                uuid!("3722746e-76a8-4131-9832-42c4783f8997").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn object_created_with_cookie() {
        let serialized = [
            55, 0, 0, 0, 44, 1, 0xb6, 0x6c, 0x03, 0x07, 0x85, 0xcf, 0x4a, 0x7c, 0x8a, 0xcf, 0x6e,
            0x99, 0xd0, 0xf4, 0x07, 0xb7, 0, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc,
            0x5e, 0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49,
            0x30, 0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7,
        ];

        let msg = EmitBusEvent {
            cookie: Some(BusListenerCookie(uuid!(
                "b66c0307-85cf-4a7c-8acf-6e99d0f407b7"
            ))),
            event: BusEvent::ObjectCreated(ObjectId::new(
                uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn object_destroyed_with_cookie() {
        let serialized = [
            55, 0, 0, 0, 44, 1, 0xb6, 0x6c, 0x03, 0x07, 0x85, 0xcf, 0x4a, 0x7c, 0x8a, 0xcf, 0x6e,
            0x99, 0xd0, 0xf4, 0x07, 0xb7, 1, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc,
            0x5e, 0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49,
            0x30, 0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7,
        ];

        let msg = EmitBusEvent {
            cookie: Some(BusListenerCookie(uuid!(
                "b66c0307-85cf-4a7c-8acf-6e99d0f407b7"
            ))),
            event: BusEvent::ObjectDestroyed(ObjectId::new(
                uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn service_created_with_cookie() {
        let serialized = [
            87, 0, 0, 0, 44, 1, 0xb6, 0x6c, 0x03, 0x07, 0x85, 0xcf, 0x4a, 0x7c, 0x8a, 0xcf, 0x6e,
            0x99, 0xd0, 0xf4, 0x07, 0xb7, 2, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc,
            0x5e, 0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49,
            0x30, 0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7, 0xaf, 0xfc, 0xb7, 0x9b, 0xe9,
            0x9c, 0x47, 0x84, 0x95, 0xe0, 0x0d, 0xc9, 0x9f, 0xcb, 0xa8, 0xff, 0x37, 0x22, 0x74,
            0x6e, 0x76, 0xa8, 0x41, 0x31, 0x98, 0x32, 0x42, 0xc4, 0x78, 0x3f, 0x89, 0x97,
        ];

        let msg = EmitBusEvent {
            cookie: Some(BusListenerCookie(uuid!(
                "b66c0307-85cf-4a7c-8acf-6e99d0f407b7"
            ))),
            event: BusEvent::ServiceCreated(ServiceId::new(
                ObjectId::new(
                    uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                    uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
                ),
                uuid!("affcb79b-e99c-4784-95e0-0dc99fcba8ff").into(),
                uuid!("3722746e-76a8-4131-9832-42c4783f8997").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn service_destroyed_with_cookie() {
        let serialized = [
            87, 0, 0, 0, 44, 1, 0xb6, 0x6c, 0x03, 0x07, 0x85, 0xcf, 0x4a, 0x7c, 0x8a, 0xcf, 0x6e,
            0x99, 0xd0, 0xf4, 0x07, 0xb7, 3, 0x3f, 0x5e, 0xb6, 0x03, 0xce, 0x4b, 0x4c, 0x48, 0xbc,
            0x5e, 0x90, 0x90, 0x7f, 0x55, 0xab, 0x8a, 0x03, 0xdb, 0xbf, 0xa4, 0x44, 0x76, 0x49,
            0x30, 0xa4, 0xd7, 0x28, 0x32, 0x6e, 0xfd, 0x7f, 0xc7, 0xaf, 0xfc, 0xb7, 0x9b, 0xe9,
            0x9c, 0x47, 0x84, 0x95, 0xe0, 0x0d, 0xc9, 0x9f, 0xcb, 0xa8, 0xff, 0x37, 0x22, 0x74,
            0x6e, 0x76, 0xa8, 0x41, 0x31, 0x98, 0x32, 0x42, 0xc4, 0x78, 0x3f, 0x89, 0x97,
        ];

        let msg = EmitBusEvent {
            cookie: Some(BusListenerCookie(uuid!(
                "b66c0307-85cf-4a7c-8acf-6e99d0f407b7"
            ))),
            event: BusEvent::ServiceDestroyed(ServiceId::new(
                ObjectId::new(
                    uuid!("3f5eb603-ce4b-4c48-bc5e-90907f55ab8a").into(),
                    uuid!("03dbbfa4-4476-4930-a4d7-28326efd7fc7").into(),
                ),
                uuid!("affcb79b-e99c-4784-95e0-0dc99fcba8ff").into(),
                uuid!("3722746e-76a8-4131-9832-42c4783f8997").into(),
            )),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::EmitBusEvent(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum BusEventKind {
    ObjectCreated = 0,
    ObjectDestroyed = 1,
    ServiceCreated = 2,
    ServiceDestroyed = 3,
}
