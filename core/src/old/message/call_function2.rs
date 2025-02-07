use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, OptionKind};
use crate::error::SerializeError;
use crate::ids::ServiceCookie;
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_serializer::Serialize;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CallFunction2 {
    pub serial: u32,
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub version: Option<u32>,
    pub value: SerializedValue,
}

impl CallFunction2 {
    pub fn with_serialize_value<T: Serialize + ?Sized>(
        serial: u32,
        service_cookie: ServiceCookie,
        function: u32,
        version: Option<u32>,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(value)?;

        Ok(Self {
            serial,
            service_cookie,
            function,
            version,
            value,
        })
    }
}

impl MessageOps for CallFunction2 {
    fn kind(&self) -> MessageKind {
        MessageKind::CallFunction2
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::CallFunction2)?;

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.function);

        match self.version {
            None => {
                serializer.put_discriminant_u8(OptionKind::None);
            }

            Some(version) => {
                serializer.put_discriminant_u8(OptionKind::Some);
                serializer.put_varint_u32_le(version);
            }
        }

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::CallFunction2)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
        let function = deserializer.try_get_varint_u32_le()?;

        let version = match deserializer.try_get_discriminant_u8()? {
            OptionKind::None => None,
            OptionKind::Some => deserializer.try_get_varint_u32_le().map(Some)?,
        };

        let value = deserializer.finish()?;

        Ok(Self {
            serial,
            service_cookie,
            function,
            version,
            value,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }
}

impl Sealed for CallFunction2 {}

impl From<CallFunction2> for Message {
    fn from(msg: CallFunction2) -> Self {
        Self::CallFunction2(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::CallFunction2;
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn call_function2_without_version() {
        let serialized = [
            30, 0, 0, 0, 62, 2, 0, 0, 0, 3, 4, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 2, 0,
        ];
        let value = 4u8;

        let msg = CallFunction2::with_serialize_value(
            1,
            ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            2,
            None,
            &value,
        )
        .unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CallFunction2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn call_function2_with_version() {
        let serialized = [
            31, 0, 0, 0, 62, 2, 0, 0, 0, 3, 4, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 2, 1, 3,
        ];
        let value = 4u8;

        let msg = CallFunction2::with_serialize_value(
            1,
            ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            2,
            Some(3),
            &value,
        )
        .unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CallFunction2(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
