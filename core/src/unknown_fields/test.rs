use super::UnknownFields;
use crate::tags::{self, PrimaryTag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, SerializedValue,
    Serializer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Old {
    field1: u32,
    fallback: UnknownFields,
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
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(1u32, &self.field1)?;
        serializer.serialize_unknown_fields(&self.fallback)?;

        serializer.finish()
    }
}

impl Deserialize<tags::Value> for Old {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut field1 = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.id() {
                1 => field1 = deserializer.deserialize::<tags::U32, _>().map(Some)?,
                _ => deserializer.add_to_unknown_fields()?,
            }
        }

        deserializer.finish_with(|fallback| {
            Ok(Self {
                field1: field1.ok_or(DeserializeError::InvalidSerialization)?,
                fallback,
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct New {
    field1: u32,
    field2: Option<u32>,
    fallback: UnknownFields,
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
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32, _>(1u32, &self.field1)?;

        if self.field2.is_some() {
            serializer.serialize::<tags::Option<tags::U32>, _>(2u32, &self.field2)?;
        }

        serializer.serialize_unknown_fields(&self.fallback)?;
        serializer.finish()
    }
}

impl Deserialize<tags::Value> for New {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut field1 = None;
        let mut field2 = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.id() {
                1 => field1 = deserializer.deserialize::<tags::U32, _>().map(Some)?,
                2 => field2 = deserializer.deserialize::<tags::Option<tags::U32>, _>()?,
                _ => deserializer.add_to_unknown_fields()?,
            }
        }

        deserializer.finish_with(|fallback| {
            Ok(Self {
                field1: field1.ok_or(DeserializeError::InvalidSerialization)?,
                field2,
                fallback,
            })
        })
    }
}

#[test]
fn old_new_old_roundtrip() {
    let old = Old {
        field1: 1,
        fallback: UnknownFields::new(),
    };

    let serialized = SerializedValue::serialize(&old).unwrap();

    let new = serialized.deserialize::<New>().unwrap();
    assert_eq!(new.field1, 1);
    assert_eq!(new.field2, None);
    assert!(new.fallback.is_empty());

    let serialized = SerializedValue::serialize(&new).unwrap();

    let old2 = serialized.deserialize::<Old>().unwrap();
    assert_eq!(old2, old);
}

#[test]
fn new_old_new_roundtrip1() {
    let new = New {
        field1: 1,
        field2: None,
        fallback: UnknownFields::new(),
    };

    let serialized = SerializedValue::serialize(&new).unwrap();

    let old = serialized.deserialize::<Old>().unwrap();
    assert_eq!(old.field1, 1);
    assert!(old.fallback.is_empty());
    assert!(!old.fallback.has_fields_set());

    let serialized = SerializedValue::serialize(&old).unwrap();

    let new2 = serialized.deserialize::<New>().unwrap();
    assert_eq!(new2, new);
}

#[test]
fn new_old_new_roundtrip2() {
    let new = New {
        field1: 1,
        field2: Some(2),
        fallback: UnknownFields::new(),
    };

    let serialized = SerializedValue::serialize(&new).unwrap();

    let old = serialized.deserialize::<Old>().unwrap();
    assert_eq!(old.field1, 1);
    assert_eq!(old.fallback.len(), 1);
    assert!(old.fallback.has_fields_set());

    let serialized = SerializedValue::serialize(&old).unwrap();

    let new2 = serialized.deserialize::<New>().unwrap();
    assert_eq!(new2, new);
}
