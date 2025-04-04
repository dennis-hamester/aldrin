use super::UnknownVariant;
use crate::tags::{self, PrimaryTag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, SerializedValue,
    Serializer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Old {
    Var1,
    Fallback(UnknownVariant),
}

impl PrimaryTag for Old {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for Old {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Value, _>(&self)
    }
}

impl Serialize<tags::Value> for &Old {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Old::Var1 => serializer.serialize_unit_enum(1u32),
            Old::Fallback(fallback) => serializer.serialize_unknown_variant(fallback),
        }
    }
}

impl Deserialize<tags::Value> for Old {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.id() {
            1 => deserializer.deserialize_unit().map(|()| Self::Var1),
            _ => deserializer.into_unknown_variant().map(Self::Fallback),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum New {
    Var1,
    Var2(u8),
    Fallback(UnknownVariant),
}

impl PrimaryTag for New {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for New {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Value, _>(&self)
    }
}

impl Serialize<tags::Value> for &New {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            New::Var1 => serializer.serialize_unit_enum(1u32),
            New::Var2(value) => serializer.serialize_enum::<tags::U8, _>(2u32, value),
            New::Fallback(fallback) => serializer.serialize_unknown_variant(fallback),
        }
    }
}

impl Deserialize<tags::Value> for New {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.id() {
            1 => deserializer.deserialize_unit().map(|()| Self::Var1),
            2 => deserializer.deserialize::<tags::U8, _>().map(Self::Var2),
            _ => deserializer.into_unknown_variant().map(Self::Fallback),
        }
    }
}

#[test]
fn old_new_old_roundtrip() {
    let old = Old::Var1;
    let serialized = SerializedValue::serialize(&old).unwrap();

    let new = serialized.deserialize::<New>().unwrap();
    assert_eq!(new, New::Var1);

    let serialized = SerializedValue::serialize(&new).unwrap();

    let old2 = serialized.deserialize::<Old>().unwrap();
    assert_eq!(old2, old);
}

#[test]
fn new_old_new_roundtrip1() {
    let new = New::Var1;
    let serialized = SerializedValue::serialize(&new).unwrap();

    let old = serialized.deserialize::<Old>().unwrap();
    assert_eq!(old, Old::Var1);

    let serialized = SerializedValue::serialize(&old).unwrap();

    let new2 = serialized.deserialize::<New>().unwrap();
    assert_eq!(new2, new);
}

#[test]
fn new_old_new_roundtrip2() {
    let new = New::Var2(1);
    let serialized = SerializedValue::serialize(&new).unwrap();

    let old = serialized.deserialize::<Old>().unwrap();

    let Old::Fallback(ref fallback) = old else {
        panic!();
    };

    assert_eq!(fallback.id(), 2);
    assert_eq!(fallback.deserialize::<u8>().unwrap(), 1u8);

    let serialized = SerializedValue::serialize(&old).unwrap();

    let new2 = serialized.deserialize::<New>().unwrap();
    assert_eq!(new2, new);
}
