use super::LexicalId;
use crate::tags::{PrimaryTag, Tag};
use crate::{Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

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

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MapTypeField {
    Key = 0,
    Value = 1,
}

impl Tag for MapTypeIr {}

impl PrimaryTag for MapTypeIr {
    type Tag = Self;
}

impl Serialize<MapTypeIr> for &MapTypeIr {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<LexicalId, _>(MapTypeField::Key, &self.key)?;
        serializer.serialize::<LexicalId, _>(MapTypeField::Value, &self.value)?;

        serializer.finish()
    }
}
