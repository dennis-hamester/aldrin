use crate::error::DeserializeError;
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVariant {
    id: u32,
    value: SerializedValue,
}

impl UnknownVariant {
    pub fn new(id: u32, value: SerializedValue) -> Self {
        Self { id, value }
    }

    pub fn id(&self) -> u32 {
        self.id
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
