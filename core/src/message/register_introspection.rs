use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::{SerializedValue, SerializedValueSlice};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct RegisterIntrospection {
    pub value: SerializedValue,
}

impl MessageOps for RegisterIntrospection {
    fn kind(&self) -> MessageKind {
        MessageKind::RegisterIntrospection
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        MessageSerializer::with_value(self.value, MessageKind::RegisterIntrospection)?.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let value =
            MessageWithValueDeserializer::new(buf, MessageKind::RegisterIntrospection)?.finish()?;

        Ok(Self { value })
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        Some(&self.value)
    }

    fn value_mut(&mut self) -> Option<&mut SerializedValue> {
        Some(&mut self.value)
    }
}

impl Sealed for RegisterIntrospection {}

impl From<RegisterIntrospection> for Message {
    fn from(msg: RegisterIntrospection) -> Self {
        Self::RegisterIntrospection(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{assert_deserialize_eq_with_value, assert_serialize_eq};
    use super::super::Message;
    use super::RegisterIntrospection;
    use crate::{tags, SerializedValue, TypeId};
    use std::collections::HashSet;
    use std::iter;
    use uuid::uuid;

    #[test]
    fn register_introspection() {
        let serialized = [
            27, 0, 0, 0, 49, 18, 0, 0, 0, 38, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb,
        ];

        let value = HashSet::<_>::from_iter(iter::once(TypeId(uuid!(
            "026c3142-530b-4d65-850d-a297dcc2fecb"
        ))));

        let msg = RegisterIntrospection {
            value: SerializedValue::serialize(&value).unwrap(),
        };

        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::Set<TypeId>, _>(&msg, serialized, &value);

        let msg = Message::RegisterIntrospection(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value::<_, _, tags::Set<TypeId>, _>(&msg, serialized, &value);
    }
}
