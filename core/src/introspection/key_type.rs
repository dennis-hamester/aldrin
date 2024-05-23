use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum KeyType {
    U8 = 0,
    I8 = 1,
    U16 = 2,
    I16 = 3,
    U32 = 4,
    I32 = 5,
    U64 = 6,
    I64 = 7,
    String = 8,
    Uuid = 9,
}

impl Serialize for KeyType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_enum(*self, &())
    }
}

impl Deserialize for KeyType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;
        let variant = deserializer.try_variant()?;
        deserializer.deserialize().map(|()| variant)
    }
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::U8 => write!(f, "u8"),
            Self::I8 => write!(f, "i8"),
            Self::U16 => write!(f, "u16"),
            Self::I16 => write!(f, "i16"),
            Self::U32 => write!(f, "u32"),
            Self::I32 => write!(f, "i32"),
            Self::U64 => write!(f, "u64"),
            Self::I64 => write!(f, "i64"),
            Self::String => write!(f, "string"),
            Self::Uuid => write!(f, "uuid"),
        }
    }
}
