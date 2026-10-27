use super::LexicalId;
use crate::enum_repr::enum_with_repr;
use crate::tags::{PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};

#[derive(Debug, Copy, Clone)]
pub struct MapTypeIr {
    pub(crate) key: LexicalId,
    pub(crate) value: LexicalId,
}

impl MapTypeIr {
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

enum_with_repr! {
    enum MapTypeField: u32 {
        Key = 0,
        Value = 1,
    }
}

impl Tag for MapTypeIr {}

impl PrimaryTag for MapTypeIr {
    type Tag = Self;
}

impl Serialize<MapTypeIr> for &MapTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId>(MapTypeField::Key, &self.key)?;
        serializer.serialize::<LexicalId>(MapTypeField::Value, &self.value)?;

        serializer.finish()
    }
}
