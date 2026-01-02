#[cfg(feature = "introspection")]
use crate::introspection::{Introspectable, LexicalId, References, ir};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use std::collections::{LinkedList, VecDeque};
use std::mem::MaybeUninit;
use std::ptr;

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
        let mut arr = [MaybeUninit::uninit(); N];
        let mut done = 0;

        loop {
            let slice = deserializer.as_slice()?;
            let len = slice.len();

            if len == 0 {
                if done == N {
                    // SAFETY: Exactly done elements have been initialized and done equals N.
                    //
                    // This is a convoluted transmute. Use MaybeUninit::array_assume_init() when
                    // it's stable: https://github.com/rust-lang/rust/issues/96097
                    let value = unsafe {
                        let arr = MaybeUninit::new(arr);
                        let arr = ptr::from_ref(&arr).cast::<MaybeUninit<[u8; N]>>();
                        let arr = &*arr;

                        arr.assume_init_read()
                    };

                    break deserializer.finish(value);
                } else {
                    break Err(DeserializeError::InvalidSerialization);
                }
            }

            if done + len <= N {
                // SAFETY: &[u8] and &[MaybeUninit<u8>] have the same layout
                //
                // Use [MaybeUninit<T>]::write_copy_of_slice() when it's stable:
                // https://github.com/rust-lang/rust/issues/79995
                let slice = unsafe {
                    let slice = ptr::from_ref(slice) as *const [MaybeUninit<u8>];
                    &*slice
                };

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
        serializer.serialize::<tags::Bytes>(&self)
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
        serializer.serialize::<tags::Bytes>(&self)
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
