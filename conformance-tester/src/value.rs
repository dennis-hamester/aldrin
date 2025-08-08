use aldrin_core::tags::{self, PrimaryTag};
use aldrin_core::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, ValueKind,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "value-type", content = "value")]
pub(crate) enum Value {
    None,
    I32(i32),
    Ignore,

    #[serde(skip_deserializing)]
    Unsupported {
        #[serde(rename = "value-type", serialize_with = "serialize_value_kind")]
        kind: ValueKind,
        serialized: Vec<u8>,
    },
}

impl Value {
    pub(crate) fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) | (Self::Ignore, _) | (_, Self::Ignore) => true,
            (Self::I32(v1), Self::I32(v2)) => v1 == v2,
            _ => false,
        }
    }
}

impl PrimaryTag for Value {
    type Tag = tags::Value;
}

impl Serialize<tags::Value> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<tags::Value> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::None | Value::Ignore => serializer.serialize_none()?,
            Value::I32(value) => serializer.serialize_i32(*value)?,
            Value::Unsupported { .. } => unreachable!(),
        }

        Ok(())
    }
}

impl Deserialize<tags::Value> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|_| Self::None),
            ValueKind::I32 => deserializer.deserialize_i32().map(Self::I32),

            kind => deserializer
                .split_off_serialized_value()
                .map(|serialized| Self::Unsupported {
                    kind,
                    serialized: serialized.to_vec(),
                }),
        }
    }
}

fn serialize_value_kind<S>(kind: &ValueKind, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let kind_str = match kind {
        ValueKind::None => "none",
        ValueKind::Some => "some",
        ValueKind::Bool => "bool",
        ValueKind::U8 => "u8",
        ValueKind::I8 => "i8",
        ValueKind::U16 => "u16",
        ValueKind::I16 => "i16",
        ValueKind::U32 => "u32",
        ValueKind::I32 => "i32",
        ValueKind::U64 => "u64",
        ValueKind::I64 => "i64",
        ValueKind::F32 => "f32",
        ValueKind::F64 => "f64",
        ValueKind::String => "string",
        ValueKind::Uuid => "uuid",
        ValueKind::ObjectId => "object-id",
        ValueKind::ServiceId => "service-id",
        ValueKind::Vec1 => "vec1",
        ValueKind::Bytes1 => "bytes1",
        ValueKind::U8Map1 => "u8-map1",
        ValueKind::I8Map1 => "i8-map1",
        ValueKind::U16Map1 => "u16-map1",
        ValueKind::I16Map1 => "i16-map1",
        ValueKind::U32Map1 => "u32-map1",
        ValueKind::I32Map1 => "i32-map1",
        ValueKind::U64Map1 => "u64-map1",
        ValueKind::I64Map1 => "i64-map1",
        ValueKind::StringMap1 => "string-map1",
        ValueKind::UuidMap1 => "uuid-map1",
        ValueKind::U8Set1 => "u8-set1",
        ValueKind::I8Set1 => "i8-set1",
        ValueKind::U16Set1 => "u16-set1",
        ValueKind::I16Set1 => "i16-set1",
        ValueKind::U32Set1 => "u32-set1",
        ValueKind::I32Set1 => "i32-set1",
        ValueKind::U64Set1 => "u64-set1",
        ValueKind::I64Set1 => "i64-set1",
        ValueKind::StringSet1 => "string-set1",
        ValueKind::UuidSet1 => "uuid-set1",
        ValueKind::Struct1 => "struct1",
        ValueKind::Enum => "enum",
        ValueKind::Sender => "sender",
        ValueKind::Receiver => "receiver",
        ValueKind::Vec2 => "vec2",
        ValueKind::Bytes2 => "bytes2",
        ValueKind::U8Map2 => "u8-map2",
        ValueKind::I8Map2 => "i8-map2",
        ValueKind::U16Map2 => "u16-map2",
        ValueKind::I16Map2 => "i16-map2",
        ValueKind::U32Map2 => "u32-map2",
        ValueKind::I32Map2 => "i32-map2",
        ValueKind::U64Map2 => "u64-map2",
        ValueKind::I64Map2 => "i64-map2",
        ValueKind::StringMap2 => "string-map2",
        ValueKind::UuidMap2 => "uuid-map2",
        ValueKind::U8Set2 => "u8-set2",
        ValueKind::I8Set2 => "i8-set2",
        ValueKind::U16Set2 => "u16-set2",
        ValueKind::I16Set2 => "i16-set2",
        ValueKind::U32Set2 => "u32-set2",
        ValueKind::I32Set2 => "i32-set2",
        ValueKind::U64Set2 => "u64-set2",
        ValueKind::I64Set2 => "i64-set2",
        ValueKind::StringSet2 => "string-set2",
        ValueKind::UuidSet2 => "uuid-set2",
        ValueKind::Struct2 => "struct2",
    };

    serializer.serialize_str(kind_str)
}
