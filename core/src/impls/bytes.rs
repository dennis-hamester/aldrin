#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use std::collections::{LinkedList, VecDeque};
use std::mem::{self, MaybeUninit};

impl Serialize<tags::Bytes> for Vec<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(&self)
    }
}

impl Serialize<tags::Bytes> for &Vec<u8> {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(self)
    }
}

impl Deserialize<tags::Bytes> for Vec<u8> {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes_extend_new()
    }
}

impl Serialize<tags::Bytes> for &[u8] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(self)
    }
}

impl<const N: usize> Serialize<tags::Bytes> for [u8; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(&self)
    }
}

impl<const N: usize> Serialize<tags::Bytes> for &[u8; N] {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(self)
    }
}

impl<const N: usize> Deserialize<tags::Bytes> for [u8; N] {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_bytes()?;

        // SAFETY: This creates an array of MaybeUninit<U>, which doesn't require initialization.
        let mut arr: [MaybeUninit<u8>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut done = 0;

        loop {
            let slice = deserializer.as_slice()?;
            let len = slice.len();

            if len == 0 {
                if done == N {
                    // SAFETY: Exactly done elements have been initialized and done equals N.
                    //
                    // It's currently impossible to transmute [MaybeUninit<u8>; N] to [u8; N] when N
                    // is a const generic. See https://github.com/rust-lang/rust/issues/61956.
                    let value = unsafe {
                        (*(&MaybeUninit::new(arr) as *const _ as *const MaybeUninit<[u8; N]>))
                            .assume_init_read()
                    };

                    break deserializer.finish(value);
                } else {
                    break Err(DeserializeError::InvalidSerialization);
                }
            }

            if done + len <= N {
                // SAFETY: &[u8] and &[MaybeUninit<u8>] have the same layout
                let slice: &[MaybeUninit<u8>] = unsafe { mem::transmute(slice) };

                arr[done..(done + len)].copy_from_slice(slice);

                done += len;
                deserializer.advance(len)?;
            } else {
                break Err(DeserializeError::InvalidSerialization);
            }
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
        let mut serializer = serializer.serialize_bytes2()?;

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
        let mut serializer = serializer.serialize_bytes1(self.len())?;

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
        serializer.serialize_byte_slice2(&self)
    }
}

impl Serialize<tags::Bytes> for &bytes::Bytes {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(self)
    }
}

impl Deserialize<tags::Bytes> for bytes::Bytes {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<tags::Bytes, bytes::BytesMut>()
            .map(bytes::BytesMut::freeze)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::Bytes {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Bytes.into()
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
        serializer.serialize_byte_slice2(&self)
    }
}

impl Serialize<tags::Bytes> for &bytes::BytesMut {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_byte_slice2(self)
    }
}

impl Deserialize<tags::Bytes> for bytes::BytesMut {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_bytes()?;
        let mut bytes = Self::new();

        loop {
            let slice = deserializer.as_slice()?;

            if slice.is_empty() {
                break deserializer.finish(bytes);
            } else {
                bytes.extend_from_slice(slice);
                deserializer.advance(slice.len())?;
            }
        }
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for bytes::BytesMut {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Bytes.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::BYTES
    }

    fn add_references(_references: &mut References) {}
}

impl Serialize<tags::Bytes> for () {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bytes2()?.finish()
    }
}

impl Serialize<tags::Bytes> for &() {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bytes2()?.finish()
    }
}

impl Deserialize<tags::Bytes> for () {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bytes()?.finish(())
    }
}
