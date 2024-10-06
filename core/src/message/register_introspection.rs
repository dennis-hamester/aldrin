use super::message_ops::Sealed;
use super::{Message, MessageKind, MessageOps};
use crate::error::{DeserializeError, SerializeError};
use crate::ids::TypeId;
use crate::message_deserializer::{MessageDeserializeError, MessageWithValueDeserializer};
use crate::message_serializer::{MessageSerializeError, MessageSerializer};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::BytesMut;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct RegisterIntrospection {
    pub value: SerializedValue,
}

impl RegisterIntrospection {
    pub fn with_serialize_type_ids(type_ids: &HashSet<TypeId>) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(&SerializeTypeIds(type_ids))?;
        Ok(Self { value })
    }

    pub fn deserialize_type_ids(&self) -> Result<HashSet<TypeId>, DeserializeError> {
        self.value
            .deserialize::<DeserializeTypeIds>()
            .map(|ids| ids.0)
    }
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
}

impl Sealed for RegisterIntrospection {}

impl From<RegisterIntrospection> for Message {
    fn from(msg: RegisterIntrospection) -> Self {
        Self::RegisterIntrospection(msg)
    }
}

struct SerializeTypeIds<'a>(&'a HashSet<TypeId>);

impl Serialize for SerializeTypeIds<'_> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_set_iter(self.0.iter().map(|id| id.0))
    }
}

struct DeserializeTypeIds(HashSet<TypeId>);

impl Deserialize for DeserializeTypeIds {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_set()?;

        let mut ids = HashSet::new();
        while !deserializer.is_empty() {
            let id = deserializer.deserialize_element().map(TypeId)?;
            ids.insert(id);
        }

        deserializer.finish(Self(ids))
    }
}

#[cfg(test)]
mod test {
    use super::super::test::assert_serialize_eq;
    use super::super::Message;
    use super::RegisterIntrospection;
    use crate::ids::TypeId;
    use uuid::uuid;

    #[test]
    fn emit_event() {
        let serialized = [
            27, 0, 0, 0, 49, 18, 0, 0, 0, 38, 1, 0x02, 0x6c, 0x31, 0x42, 0x53, 0x0b, 0x4d, 0x65,
            0x85, 0x0d, 0xa2, 0x97, 0xdc, 0xc2, 0xfe, 0xcb,
        ];
        let ids = Some(TypeId(uuid!("026c3142-530b-4d65-850d-a297dcc2fecb")))
            .into_iter()
            .collect();

        let msg = RegisterIntrospection::with_serialize_type_ids(&ids).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_eq!(ids, msg.deserialize_type_ids().unwrap());

        let msg = Message::RegisterIntrospection(msg);
        assert_serialize_eq(&msg, serialized);
    }
}
