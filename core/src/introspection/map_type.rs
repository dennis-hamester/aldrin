use super::LexicalId;
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MapType {
    key: LexicalId,
    value: LexicalId,
}

impl MapType {
    pub fn new(key: LexicalId, value: LexicalId) -> Self {
        Self { key, value }
    }

    pub fn key(self) -> LexicalId {
        self.key
    }

    pub fn value(self) -> LexicalId {
        self.value
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MapTypeField {
    Key = 0,
    Value = 1,
}

impl Tag for MapType {}

impl PrimaryTag for MapType {
    type Tag = Self;
}

impl Serialize<Self> for MapType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct1(2)?;

        serializer.serialize::<LexicalId, _>(MapTypeField::Key, self.key)?;
        serializer.serialize::<LexicalId, _>(MapTypeField::Value, self.value)?;

        serializer.finish()
    }
}

impl Serialize<MapType> for &MapType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for MapType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut key = None;
        let mut value = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(MapTypeField::Key) => {
                    key = deserializer.deserialize::<LexicalId, _>().map(Some)?
                }

                Ok(MapTypeField::Value) => {
                    value = deserializer.deserialize::<LexicalId, _>().map(Some)?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            key: key.ok_or(DeserializeError::InvalidSerialization)?,
            value: value.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
