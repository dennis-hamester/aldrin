use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{BusListenerCookie, BusListenerFilter, SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct AddBusListenerFilter {
    pub cookie: BusListenerCookie,
    pub filter: BusListenerFilter,
}

impl MessageOps for AddBusListenerFilter {
    fn kind(&self) -> MessageKind {
        MessageKind::AddBusListenerFilter
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::AddBusListenerFilter);

        serializer.put_uuid(self.cookie.0);
        self.filter.serialize_into_message(&mut serializer);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::AddBusListenerFilter)?;

        let cookie = deserializer.try_get_uuid().map(BusListenerCookie)?;
        let filter = BusListenerFilter::deserialize_from_message(&mut deserializer)?;

        deserializer.finish()?;
        Ok(Self { cookie, filter })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for AddBusListenerFilter {}

impl From<AddBusListenerFilter> for Message {
    fn from(msg: AddBusListenerFilter) -> Self {
        Self::AddBusListenerFilter(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::Message;
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::AddBusListenerFilter;
    use crate::{BusListenerCookie, BusListenerFilter, ObjectUuid, ServiceUuid};
    use uuid::uuid;

    #[test]
    fn any_object() {
        let serialized = [
            22, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 0,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::any_object(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn specific_object() {
        let serialized = [
            38, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 1, 0xb7, 0xf4, 0x93, 0x0e, 0xa6, 0x73, 0x4b, 0x3d, 0x95,
            0x45, 0x78, 0x87, 0xfa, 0x8b, 0xde, 0x3f,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::object(ObjectUuid(uuid!(
                "b7f4930e-a673-4b3d-9545-7887fa8bde3f"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn any_object_any_service() {
        let serialized = [
            22, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 2,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::any_object_any_service(),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn specific_object_any_service() {
        let serialized = [
            38, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 3, 0xb7, 0xf4, 0x93, 0x0e, 0xa6, 0x73, 0x4b, 0x3d, 0x95,
            0x45, 0x78, 0x87, 0xfa, 0x8b, 0xde, 0x3f,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::specific_object_any_service(ObjectUuid(uuid!(
                "b7f4930e-a673-4b3d-9545-7887fa8bde3f"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn any_object_specific_service() {
        let serialized = [
            38, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 4, 0xb7, 0xf4, 0x93, 0x0e, 0xa6, 0x73, 0x4b, 0x3d, 0x95,
            0x45, 0x78, 0x87, 0xfa, 0x8b, 0xde, 0x3f,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::any_object_specific_service(ServiceUuid(uuid!(
                "b7f4930e-a673-4b3d-9545-7887fa8bde3f"
            ))),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn specific_object_specific_service() {
        let serialized = [
            54, 0, 0, 0, 37, 0x89, 0xe6, 0x24, 0x38, 0x29, 0x91, 0x48, 0xf8, 0xae, 0x1d, 0x7a,
            0xd9, 0xdd, 0xcd, 0x7e, 0x72, 5, 0x8a, 0x88, 0xcf, 0xe6, 0x26, 0xae, 0x4c, 0x5a, 0x8f,
            0x70, 0x5e, 0x11, 0xbe, 0x41, 0xd2, 0x5a, 0xb7, 0xf4, 0x93, 0x0e, 0xa6, 0x73, 0x4b,
            0x3d, 0x95, 0x45, 0x78, 0x87, 0xfa, 0x8b, 0xde, 0x3f,
        ];

        let msg = AddBusListenerFilter {
            cookie: BusListenerCookie(uuid!("89e62438-2991-48f8-ae1d-7ad9ddcd7e72")),
            filter: BusListenerFilter::specific_object_and_service(
                ObjectUuid(uuid!("8a88cfe6-26ae-4c5a-8f70-5e11be41d25a")),
                ServiceUuid(uuid!("b7f4930e-a673-4b3d-9545-7887fa8bde3f")),
            ),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::AddBusListenerFilter(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
