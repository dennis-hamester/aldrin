use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps, MessageSerializer, MessageWithValueDeserializer};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceCookie;
use crate::value_serializer::Serialize;
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub value: BytesMut,
}

impl CallFunction {
    pub fn with_serialize_value<T: Serialize + ?Sized>(
        serial: u32,
        service_cookie: ServiceCookie,
        function: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = super::message_buf_with_serialize_value(value)?;
        Ok(Self {
            serial,
            service_cookie,
            function,
            value,
        })
    }

    fn value_buf(&self) -> &[u8] {
        debug_assert!(self.value.len() >= 6);
        &self.value[5..]
    }
}

impl MessageOps for CallFunction {
    fn kind(&self) -> MessageKind {
        MessageKind::CallFunction
    }

    fn serialize_message(self) -> Result<BytesMut, SerializeError> {
        let mut serializer = MessageSerializer::with_value(self.value, MessageKind::CallFunction)?;

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.service_cookie.0);
        serializer.put_varint_u32_le(self.function);

        Ok(serializer.finish())
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, DeserializeError> {
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

    fn value_buf_opt(&self) -> Option<&[u8]> {
        Some(self.value_buf())
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
    use crate::ids::ServiceCookie;
    use uuid::uuid;

    #[test]
    fn call_function() {
        let serialized = [
            21, 2, 0, 0, 0, 3, 4, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65, 0x85, 0x0d,
            0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb, 2,
        ];
        let value = 4u8;

        let msg = CallFunction::with_serialize_value(
            1,
            ServiceCookie(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")),
            2,
            &value,
        )
        .unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CallFunction(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }
}
