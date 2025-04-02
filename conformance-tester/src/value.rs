use aldrin_core::tags::{self, PrimaryTag};
use aldrin_core::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, ValueKind,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "value-type", content = "value")]
pub enum Value {
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
    pub fn matches(&self, other: &Self) -> bool {
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
        ValueKind::Bytes => "bytes",
        ValueKind::U8Map => "u8-map",
        ValueKind::I8Map => "i8-map",
        ValueKind::U16Map => "u16-map",
        ValueKind::I16Map => "i16-map",
        ValueKind::U32Map => "u32-map",
        ValueKind::I32Map => "i32-map",
        ValueKind::U64Map => "u64-map",
        ValueKind::I64Map => "i64-map",
        ValueKind::StringMap => "string-map",
        ValueKind::UuidMap => "uuid-map",
        ValueKind::U8Set => "u8-set",
        ValueKind::I8Set => "i8-set",
        ValueKind::U16Set => "u16-set",
        ValueKind::I16Set => "i16-set",
        ValueKind::U32Set => "u32-set",
        ValueKind::I32Set => "i32-set",
        ValueKind::U64Set => "u64-set",
        ValueKind::I64Set => "i64-set",
        ValueKind::StringSet => "string-set",
        ValueKind::UuidSet => "uuid-set",
        ValueKind::Struct => "struct",
        ValueKind::Enum => "enum",
        ValueKind::Sender => "sender",
        ValueKind::Receiver => "receiver",
        ValueKind::Vec2 => "vec2",
    };

    serializer.serialize_str(kind_str)
}
