use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice, ServiceCookie};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub value: SerializedValue,
}

impl MessageOps for CallFunction {
    fn kind(&self) -> MessageKind {
        MessageKind::CallFunction
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::CallFunction)?;

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.function);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer = MessageWithValueDeserializer::new(buf, MessageKind::CallFunction)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let service_cookie = deserializer.try_get_uuid().map(ServiceCookie)?;
        let function = deserializer.try_get_varint_u32_le()?;
        let value = deserializer.finish()?;

        Ok(Self {
            serial,
            service_cookie,
            function,
            value,
        })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
    }
}

impl Sealed for CallFunction {}

impl From<CallFunction> for Message {
    fn from(msg: CallFunction) -> Self {
        Self::CallFunction(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::CallFunction;
    use crate::{tags, SerializedValue, ServiceCookie};
    use uuid::uuid;

    #[test]
    fn call_function() {
        let serialized = [
            29, 0, 0, 0, 11, 2, 0, 0, 0, 3, 4, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 2,
        ];
        let value = 4u8;

        let msg = CallFunction {
            serial: 1,
            service_cookie: ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            function: 2,
            value: SerializedValue::serialize(value).unwrap(),
        };

        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::U8, _>(&msg, serialized, &value);

        let msg = Message::CallFunction(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::U8, _>(&msg, serialized, &value);
    }
}
