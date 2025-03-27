use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithoutValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice, TypeId};
use bytes::BytesMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct QueryIntrospection {
    pub serial: u32,
    pub type_id: TypeId,
}

impl MessageOps for QueryIntrospection {
    fn kind(&self) -> MessageKind {
        MessageKind::QueryIntrospection
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let mut serializer = MessageSerializer::without_value(MessageKind::QueryIntrospection);

        serializer.put_varint_u32_le(self.serial);
        serializer.put_uuid(self.type_id.0);

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithoutValueDeserializer::new(buf, MessageKind::QueryIntrospection)?;

        let serial = deserializer.try_get_varint_u32_le()?;
        let type_id = deserializer.try_get_uuid().map(TypeId)?;

        deserializer.finish()?;
        Ok(Self { serial, type_id })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        None
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        None
    }
}

impl Sealed for QueryIntrospection {}

impl From<QueryIntrospection> for Message {
    fn from(msg: QueryIntrospection) -> Self {
        Self::QueryIntrospection(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq, assert_serialize_eq};
    use super::super::Message;
    use super::QueryIntrospection;
    use crate::TypeId;
    use uuid::uuid;

    #[test]
    fn query_introspection() {
        let serialized = [
            22, 0, 0, 0, 50, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e, 0xb4, 0xbf, 0x37,
            0x38, 0x76, 0x52, 0x3d, 0x1b,
        ];

        let msg = QueryIntrospection {
            serial: 1,
            type_id: TypeId(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::QueryIntrospection(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
