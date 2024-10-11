use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::{uuid, Uuid};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive,
)]
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

impl KeyType {
    pub const U8_KEY_ID: Uuid = uuid!("368832f6-dd64-4e5c-b7bf-db5d405841dc");
    pub const I8_KEY_ID: Uuid = uuid!("b1d693de-1f02-4c03-b8ba-64a1558fd3d8");
    pub const U16_KEY_ID: Uuid = uuid!("cb82dbea-8c4a-411d-a67e-f0a054609992");
    pub const I16_KEY_ID: Uuid = uuid!("6a390f75-597e-49f5-849a-f24be8afc75d");
    pub const U32_KEY_ID: Uuid = uuid!("9b7f5a0a-fa7f-4d32-8d3f-9544a928c74d");
    pub const I32_KEY_ID: Uuid = uuid!("dfd2b6bb-b373-447d-8edb-ff84ad318162");
    pub const U64_KEY_ID: Uuid = uuid!("85125171-61be-4e0e-a85a-767bef8bd0ff");
    pub const I64_KEY_ID: Uuid = uuid!("1727505f-7b85-4cfb-b71a-dc6fced82c43");
    pub const STRING_KEY_ID: Uuid = uuid!("8b9fa4aa-94bb-47f7-9665-bc52dc63a61f");
    pub const UUID_KEY_ID: Uuid = uuid!("18ede727-5c3c-4a2b-a21f-55ba1f51ad03");

    pub fn id(self) -> Uuid {
        match self {
            Self::U8 => Self::U8_KEY_ID,
            Self::I8 => Self::I8_KEY_ID,
            Self::U16 => Self::U16_KEY_ID,
            Self::I16 => Self::I16_KEY_ID,
            Self::U32 => Self::U32_KEY_ID,
            Self::I32 => Self::I32_KEY_ID,
            Self::U64 => Self::U64_KEY_ID,
            Self::I64 => Self::I64_KEY_ID,
            Self::String => Self::STRING_KEY_ID,
            Self::Uuid => Self::UUID_KEY_ID,
        }
    }
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

pub trait Sealed {}

pub trait KeyTypeOf: Sealed {
    fn key_type_of() -> KeyType;
}

impl<T: Sealed + ?Sized> Sealed for &T {}

impl<T: KeyTypeOf + ?Sized> KeyTypeOf for &T {
    fn key_type_of() -> KeyType {
        T::key_type_of()
    }
}

impl<T: Sealed + ?Sized> Sealed for &mut T {}

impl<T: KeyTypeOf + ?Sized> KeyTypeOf for &mut T {
    fn key_type_of() -> KeyType {
        T::key_type_of()
    }
}

impl Sealed for u8 {}

impl KeyTypeOf for u8 {
    fn key_type_of() -> KeyType {
        KeyType::U8
    }
}

impl Sealed for i8 {}

impl KeyTypeOf for i8 {
    fn key_type_of() -> KeyType {
        KeyType::I8
    }
}

impl Sealed for u16 {}

impl KeyTypeOf for u16 {
    fn key_type_of() -> KeyType {
        KeyType::U16
    }
}

impl Sealed for i16 {}

impl KeyTypeOf for i16 {
    fn key_type_of() -> KeyType {
        KeyType::I16
    }
}

impl Sealed for u32 {}

impl KeyTypeOf for u32 {
    fn key_type_of() -> KeyType {
        KeyType::U32
    }
}

impl Sealed for i32 {}

impl KeyTypeOf for i32 {
    fn key_type_of() -> KeyType {
        KeyType::I32
    }
}

impl Sealed for u64 {}

impl KeyTypeOf for u64 {
    fn key_type_of() -> KeyType {
        KeyType::U64
    }
}

impl Sealed for i64 {}

impl KeyTypeOf for i64 {
    fn key_type_of() -> KeyType {
        KeyType::I64
    }
}

impl Sealed for String {}

impl KeyTypeOf for String {
    fn key_type_of() -> KeyType {
        KeyType::String
    }
}

impl Sealed for str {}

impl KeyTypeOf for str {
    fn key_type_of() -> KeyType {
        KeyType::String
    }
}

impl Sealed for Uuid {}

impl KeyTypeOf for Uuid {
    fn key_type_of() -> KeyType {
        KeyType::Uuid
    }
}
