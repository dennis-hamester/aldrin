use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use bytes::BytesMut;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Eq)]
pub struct SerializedValue {
    buf: BytesMut,
}

impl SerializedValue {
    /// Cheaply creates an empty `SerializedValue`.
    pub fn empty() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    pub fn serialize<T: Serialize + ?Sized>(value: &T) -> Result<Self, SerializeError> {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length.
        let mut buf = BytesMut::zeroed(9);
        let serializer = Serializer::new(&mut buf);
        value.serialize(serializer)?;
        Ok(Self { buf })
    }

    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        SerializedValueRef::from(self).deserialize()
    }

    pub(crate) fn from_bytes_mut(buf: BytesMut) -> Self {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length + at least 1 byte
        // value.
        debug_assert!(buf.len() >= 10);

        Self { buf }
    }

    pub(crate) fn into_bytes_mut(self) -> BytesMut {
        self.buf
    }
}

impl AsRef<[u8]> for SerializedValue {
    fn as_ref(&self) -> &[u8] {
        // 4 bytes message length + 1 byte message kind + 4 bytes value length.
        &self.buf[9..]
    }
}

impl From<SerializedValueRef<'_>> for SerializedValue {
    fn from(v: SerializedValueRef) -> Self {
        Self::serialize(&v).unwrap()
    }
}

impl PartialEq for SerializedValue {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<SerializedValueRef<'_>> for SerializedValue {
    fn eq(&self, other: &SerializedValueRef) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Serialize for SerializedValue {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        SerializedValueRef::from(self).serialize(serializer)
    }
}

impl Deserialize for SerializedValue {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.split_off_serialized_value().map(Into::into)
    }
}

#[cfg(feature = "fuzzing")]
impl<'a> arbitrary::Arbitrary<'a> for SerializedValue {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.arbitrary_len::<u8>()?;
        let bytes = u.bytes(len)?;
        if bytes.len() >= 10 {
            Ok(Self::from_bytes_mut(bytes.into()))
        } else {
            Err(arbitrary::Error::NotEnoughData)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SerializedValueRef<'a> {
    buf: &'a [u8],
}

impl<'a> SerializedValueRef<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn deserialize<T: Deserialize>(self) -> Result<T, DeserializeError> {
        let mut buf = self.buf;
        let deserializer = Deserializer::new(&mut buf);

        let res = T::deserialize(deserializer);

        if res.is_ok() && !buf.is_empty() {
            return Err(DeserializeError::TrailingData);
        }

        res
    }
}

impl AsRef<[u8]> for SerializedValueRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.buf
    }
}

impl<'a> From<&'a SerializedValue> for SerializedValueRef<'a> {
    fn from(v: &'a SerializedValue) -> Self {
        Self::new(v.as_ref())
    }
}

impl PartialEq<SerializedValue> for SerializedValueRef<'_> {
    fn eq(&self, other: &SerializedValue) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Serialize for SerializedValueRef<'_> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.copy_from_serialized_value(*self);
        Ok(())
    }
}
