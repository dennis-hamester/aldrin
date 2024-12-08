use crate::error::{DeserializeError, SerializeError};
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{AsSerializeArg, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumFallback {
    variant: u32,
    value: SerializedValue,
}

impl EnumFallback {
    pub fn new(variant: u32, value: SerializedValue) -> Self {
        Self { variant, value }
    }

    pub fn with_serialize_value<T: Serialize + ?Sized>(
        variant: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let value = SerializedValue::serialize(value)?;
        Ok(Self::new(variant, value))
    }

    pub fn variant(&self) -> u32 {
        self.variant
    }

    pub fn value(&self) -> &SerializedValueSlice {
        &self.value
    }

    pub fn into_value(self) -> SerializedValue {
        self.value
    }

    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.value.deserialize()
    }
}

impl Serialize for EnumFallback {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_enum(self.variant, &self.value)
    }
}

impl Deserialize for EnumFallback {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;
        let variant = deserializer.variant();
        let value = deserializer.deserialize()?;
        Ok(Self::new(variant, value))
    }
}

impl AsSerializeArg for EnumFallback {
    type SerializeArg<'a> = &'a Self;

    fn as_serialize_arg<'a>(&'a self) -> Self::SerializeArg<'a>
    where
        Self: 'a,
    {
        self
    }
}
