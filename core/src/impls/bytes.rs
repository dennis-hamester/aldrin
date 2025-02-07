#[cfg(feature = "introspection")]
use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use std::cmp::Ordering;
use std::collections::{LinkedList, VecDeque};

impl Serialize<tags::Bytes> for Vec<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(&self)
    }
}

impl Serialize<tags::Bytes> for &Vec<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize<tags::Bytes> for Vec<u8> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_extend_new()
    }
}

impl Serialize<tags::Bytes> for &[u8] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl<const N: usize> Serialize<tags::Bytes> for [u8; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(&self)
    }
}

impl<const N: usize> Serialize<tags::Bytes> for &[u8; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl<const N: usize> Deserialize<tags::Bytes> for [u8; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_bytes()?;
        let bytes = deserializer.as_slice();

        match bytes.len().cmp(&N) {
            Ordering::Equal => {
                let bytes = bytes.try_into().unwrap();
                deserializer.advance(N)?;
                deserializer.finish(bytes)
            }

            Ordering::Less => Err(DeserializeError::NoMoreElements),
            Ordering::Greater => Err(DeserializeError::MoreElementsRemain),
        }
    }
}

impl Serialize<tags::Bytes> for VecDeque<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Bytes, _>(&self)
    }
}

impl Serialize<tags::Bytes> for &VecDeque<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_bytes(self.len())?;

        let (a, b) = self.as_slices();
        serializer.serialize(a)?;
        serializer.serialize(b)?;

        serializer.finish()
    }
}

impl Deserialize<tags::Bytes> for VecDeque<u8> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_extend_new()
    }
}

impl Serialize<tags::Bytes> for LinkedList<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Bytes, _>(&self)
    }
}

impl Serialize<tags::Bytes> for &LinkedList<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_bytes(self.len())?;

        for byte in self {
            serializer.serialize(&[*byte])?;
        }

        serializer.finish()
    }
}

impl Deserialize<tags::Bytes> for LinkedList<u8> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_extend_new()
    }
}

impl PrimaryTag for bytes::Bytes {
    type Tag = tags::Bytes;
}

impl Serialize<tags::Bytes> for bytes::Bytes {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(&self)
    }
}

impl Serialize<tags::Bytes> for &bytes::Bytes {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize<tags::Bytes> for bytes::Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_bytes()?;

        let bytes = deserializer.as_slice();
        let bytes = Self::copy_from_slice(bytes);
        deserializer.advance(bytes.len())?;

        deserializer.finish(bytes)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::Bytes {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

impl PrimaryTag for bytes::BytesMut {
    type Tag = tags::Bytes;
}

impl Serialize<tags::Bytes> for bytes::BytesMut {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(&self)
    }
}

impl Serialize<tags::Bytes> for &bytes::BytesMut {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice(self)
    }
}

impl Deserialize<tags::Bytes> for bytes::BytesMut {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_bytes()?;

        let bytes = Self::from(deserializer.as_slice());
        deserializer.advance(bytes.len())?;

        deserializer.finish(bytes)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::BytesMut {
    fn layout() -> Layout {
        BuiltInType::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

impl Serialize<tags::Bytes> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bytes(0)?.finish()
    }
}

impl Serialize<tags::Bytes> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bytes(0)?.finish()
    }
}

impl Deserialize<tags::Bytes> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes()?.finish(())
    }
}
