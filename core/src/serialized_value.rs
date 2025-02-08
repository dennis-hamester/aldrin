use bytes::BytesMut;
use std::borrow::Borrow;
use std::fmt;
use std::mem;
use std::ops::Deref;
// #[cfg(feature = "introspection")]
// use crate::introspection::{BuiltInType, Introspectable, Layout, LexicalId, References};

// 4 bytes message length + 1 byte message kind + 4 bytes value length.
const MSG_HEADER_LEN: usize = 9;

#[derive(Clone, Eq)]
pub struct SerializedValue {
    buf: BytesMut,
}

impl SerializedValue {
    /// Cheaply creates an empty `SerializedValue`.
    ///
    /// Note that an empty `SerializedValue` will panic when derefencing it to a
    /// [`SerializedValueSlice`] and when trying to deserialize it.
    pub fn empty() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    // pub fn serialize<T: Serialize + ?Sized>(value: &T) -> Result<Self, SerializeError> {
    //     // 4 bytes message length + 1 byte message kind + 4 bytes value length.
    //     let mut buf = BytesMut::zeroed(MSG_HEADER_LEN);
    //     let serializer = Serializer::new(&mut buf, 0)?;
    //     value.serialize(serializer)?;
    //     Ok(Self { buf })
    // }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    pub(crate) fn from_bytes_mut(buf: BytesMut) -> Self {
        debug_assert!(buf.len() > MSG_HEADER_LEN);
        Self { buf }
    }

    // pub(crate) fn into_bytes_mut(self) -> BytesMut {
    //     self.buf
    // }
}

impl Default for SerializedValue {
    fn default() -> Self {
        Self::empty()
    }
}

impl Deref for SerializedValue {
    type Target = SerializedValueSlice;

    fn deref(&self) -> &SerializedValueSlice {
        SerializedValueSlice::new(&self.buf[MSG_HEADER_LEN..])
    }
}

// The default Debug implementation renders the bytes as ASCII or escape sequences, which isn't
// particularly useful here.
impl fmt::Debug for SerializedValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SerializedValue")
            .field("buf", &&*self.buf)
            .finish()
    }
}

impl AsRef<SerializedValueSlice> for SerializedValue {
    fn as_ref(&self) -> &SerializedValueSlice {
        self
    }
}

impl Borrow<SerializedValueSlice> for SerializedValue {
    fn borrow(&self) -> &SerializedValueSlice {
        self
    }
}

impl AsRef<[u8]> for SerializedValue {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl PartialEq for SerializedValue {
    fn eq(&self, other: &Self) -> bool {
        ***self == ***other
    }
}

impl PartialEq<SerializedValueSlice> for SerializedValue {
    fn eq(&self, other: &SerializedValueSlice) -> bool {
        ***self == **other
    }
}

impl PartialEq<[u8]> for SerializedValue {
    fn eq(&self, other: &[u8]) -> bool {
        ***self == *other
    }
}

impl PartialEq<SerializedValue> for [u8] {
    fn eq(&self, other: &SerializedValue) -> bool {
        *self == ***other
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for SerializedValue {
//     fn layout() -> Layout {
//         BuiltInType::Value.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::VALUE
//     }

//     fn add_references(_references: &mut References) {}
// }

#[cfg(feature = "fuzzing")]
impl<'a> arbitrary::Arbitrary<'a> for SerializedValue {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.arbitrary_len::<u8>()?.max(1);
        let bytes = u.bytes(len)?;

        let mut buf = BytesMut::with_capacity(MSG_HEADER_LEN + len);
        buf.resize(MSG_HEADER_LEN, 0);
        buf.extend_from_slice(bytes);

        Ok(Self::from_bytes_mut(buf))
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SerializedValueSlice([u8]);

impl SerializedValueSlice {
    pub(crate) fn new<T: AsRef<[u8]> + ?Sized>(buf: &T) -> &Self {
        let self_ptr = buf.as_ref() as *const [u8] as *const Self;
        // Safe because of repr(transparent).
        unsafe { &*self_ptr }
    }

    // pub fn kind(&self) -> Result<ValueKind, DeserializeError> {
    //     let mut buf = &self.0;
    //     let deserializer = Deserializer::new(&mut buf, 0)?;
    //     deserializer.peek_value_kind()
    // }

    // pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
    //     let mut buf = &self.0;
    //     let deserializer = Deserializer::new(&mut buf, 0)?;

    //     let res = T::deserialize(deserializer);

    //     if res.is_ok() && !buf.is_empty() {
    //         return Err(DeserializeError::TrailingData);
    //     }

    //     res
    // }

    // pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
    //     self.deserialize()
    // }
}

impl Deref for SerializedValueSlice {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SerializedValueSlice {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl ToOwned for SerializedValueSlice {
    type Owned = SerializedValue;

    fn to_owned(&self) -> SerializedValue {
        let mut buf = BytesMut::with_capacity(MSG_HEADER_LEN + self.len());
        buf.resize(MSG_HEADER_LEN, 0);
        buf.extend_from_slice(self);

        SerializedValue::from_bytes_mut(buf)
    }
}

impl PartialEq<SerializedValue> for SerializedValueSlice {
    fn eq(&self, other: &SerializedValue) -> bool {
        **self == ***other
    }
}

impl PartialEq<[u8]> for SerializedValueSlice {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl PartialEq<SerializedValueSlice> for [u8] {
    fn eq(&self, other: &SerializedValueSlice) -> bool {
        *self == **other
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for SerializedValueSlice {
//     fn layout() -> Layout {
//         BuiltInType::Value.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::VALUE
//     }

//     fn add_references(_references: &mut References) {}
// }
