#[cfg(test)]
mod test;

#[cfg(feature = "introspection")]
use crate::introspection::{ir, Introspectable, LexicalId, References};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    convert_value, Deserialize, DeserializeError, Deserializer, ProtocolVersion, Serialize,
    SerializeError, Serializer, Value, ValueConversionError, ValueKind,
};
use bytes::BytesMut;
use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::ops::Deref;
use std::{fmt, mem};

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

    pub fn serialize_as<T: Tag>(value: impl Serialize<T>) -> Result<Self, SerializeError> {
        let mut this = Self::new();

        let serializer = Serializer::new(&mut this.buf, 0)?;
        value.serialize(serializer)?;

        Ok(this)
    }

    pub fn serialize<T>(value: T) -> Result<Self, SerializeError>
    where
        T: PrimaryTag + Serialize<T::Tag>,
    {
        Self::serialize_as(value)
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    pub fn convert(
        &mut self,
        from: Option<ProtocolVersion>,
        to: ProtocolVersion,
    ) -> Result<(), ValueConversionError> {
        convert_value::convert_mut(self, from, to)
    }

    pub(crate) fn new() -> Self {
        Self {
            buf: BytesMut::zeroed(MSG_HEADER_LEN),
        }
    }

    pub(crate) fn from_bytes_mut(buf: BytesMut) -> Self {
        debug_assert!(buf.len() > MSG_HEADER_LEN);
        Self { buf }
    }

    pub(crate) fn into_bytes_mut(self) -> BytesMut {
        self.buf
    }
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

impl PartialOrd for SerializedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SerializedValue {
    fn cmp(&self, other: &Self) -> Ordering {
        (***self).cmp(&***other)
    }
}

impl PartialOrd<SerializedValueSlice> for SerializedValue {
    fn partial_cmp(&self, other: &SerializedValueSlice) -> Option<Ordering> {
        (***self).partial_cmp(&**other)
    }
}

impl PartialOrd<[u8]> for SerializedValue {
    fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
        (***self).partial_cmp(other)
    }
}

impl PartialOrd<SerializedValue> for [u8] {
    fn partial_cmp(&self, other: &SerializedValue) -> Option<Ordering> {
        (*self).partial_cmp(&***other)
    }
}

impl PrimaryTag for SerializedValue {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for SerializedValue {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<tags::Value> for &SerializedValue {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&**self)
    }
}

impl Deserialize<tags::Value> for SerializedValue {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .split_off_serialized_value()
            .map(SerializedValueSlice::to_owned)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for SerializedValue {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Value.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::VALUE
    }

    fn add_references(_references: &mut References) {}
}

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SerializedValueSlice([u8]);

impl SerializedValueSlice {
    pub(crate) fn new<T: AsRef<[u8]> + ?Sized>(buf: &T) -> &Self {
        let self_ptr = buf.as_ref() as *const [u8] as *const Self;
        // Safe because of repr(transparent).
        unsafe { &*self_ptr }
    }

    pub fn kind(&self) -> Result<ValueKind, DeserializeError> {
        let mut buf = &self.0;
        let deserializer = Deserializer::new(&mut buf, 0)?;
        deserializer.peek_value_kind()
    }

    pub fn deserialize_as<T: Tag, U: Deserialize<T>>(&self) -> Result<U, DeserializeError> {
        let mut buf = &self.0;
        let deserializer = Deserializer::new(&mut buf, 0)?;

        let res = U::deserialize(deserializer);

        if res.is_ok() && !buf.is_empty() {
            return Err(DeserializeError::TrailingData);
        }

        res
    }

    pub fn deserialize<T: PrimaryTag + Deserialize<T::Tag>>(&self) -> Result<T, DeserializeError> {
        self.deserialize_as()
    }

    pub fn deserialize_as_value(&self) -> Result<Value, DeserializeError> {
        self.deserialize()
    }

    pub fn convert(
        &self,
        from: Option<ProtocolVersion>,
        to: ProtocolVersion,
    ) -> Result<Cow<'_, Self>, ValueConversionError> {
        convert_value::convert(self, from, to)
    }
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

impl PartialOrd<SerializedValue> for SerializedValueSlice {
    fn partial_cmp(&self, other: &SerializedValue) -> Option<Ordering> {
        (**self).partial_cmp(&***other)
    }
}

impl PartialOrd<[u8]> for SerializedValueSlice {
    fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
        (**self).partial_cmp(other)
    }
}

impl PartialOrd<SerializedValueSlice> for [u8] {
    fn partial_cmp(&self, other: &SerializedValueSlice) -> Option<Ordering> {
        (*self).partial_cmp(&**other)
    }
}

impl PrimaryTag for &SerializedValueSlice {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for &SerializedValueSlice {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.copy_from_serialized_value(self)
    }
}

#[cfg(feature = "introspection")]
impl Introspectable for SerializedValueSlice {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Value.into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::VALUE
    }

    fn add_references(_references: &mut References) {}
}
