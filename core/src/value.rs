use crate::{
    ChannelCookie, Deserialize, DeserializeError, Deserializer, ObjectId, PrimaryTag, Receiver,
    Sender, Serialize, SerializeError, Serializer, ServiceId, Tag, ValueKind,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Value {
    None,
    Some(Box<Self>),
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Uuid(Uuid),
    ObjectId(ObjectId),
    ServiceId(ServiceId),
    Vec(Vec<Self>),
    // Bytes(Bytes),
    // U8Map(HashMap<u8, Self>),
    // I8Map(HashMap<i8, Self>),
    // U16Map(HashMap<u16, Self>),
    // I16Map(HashMap<i16, Self>),
    // U32Map(HashMap<u32, Self>),
    // I32Map(HashMap<i32, Self>),
    // U64Map(HashMap<u64, Self>),
    // I64Map(HashMap<i64, Self>),
    // StringMap(HashMap<String, Self>),
    // UuidMap(HashMap<Uuid, Self>),
    // U8Set(HashSet<u8>),
    // I8Set(HashSet<i8>),
    // U16Set(HashSet<u16>),
    // I16Set(HashSet<i16>),
    // U32Set(HashSet<u32>),
    // I32Set(HashSet<i32>),
    // U64Set(HashSet<u64>),
    // I64Set(HashSet<i64>),
    // StringSet(HashSet<String>),
    // UuidSet(HashSet<Uuid>),
    // Struct(Struct),
    // Enum(Box<Enum>),
    Sender(ChannelCookie),
    Receiver(ChannelCookie),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn kind(&self) -> ValueKind {
        match self {
            Self::None => ValueKind::None,
            Self::Some(_) => ValueKind::Some,
            Self::Bool(_) => ValueKind::Bool,
            Self::U8(_) => ValueKind::U8,
            Self::I8(_) => ValueKind::I8,
            Self::U16(_) => ValueKind::U16,
            Self::I16(_) => ValueKind::I16,
            Self::U32(_) => ValueKind::U32,
            Self::I32(_) => ValueKind::I32,
            Self::U64(_) => ValueKind::U64,
            Self::I64(_) => ValueKind::I64,
            Self::F32(_) => ValueKind::F32,
            Self::F64(_) => ValueKind::F64,
            Self::String(_) => ValueKind::String,
            Self::Uuid(_) => ValueKind::Uuid,
            Self::ObjectId(_) => ValueKind::ObjectId,
            Self::ServiceId(_) => ValueKind::ServiceId,
            Self::Vec(_) => ValueKind::Vec,
            // Self::Bytes(_) => ValueKind::Bytes,
            // Self::U8Map(_) => ValueKind::U8Map,
            // Self::I8Map(_) => ValueKind::I8Map,
            // Self::U16Map(_) => ValueKind::U16Map,
            // Self::I16Map(_) => ValueKind::I16Map,
            // Self::U32Map(_) => ValueKind::U32Map,
            // Self::I32Map(_) => ValueKind::I32Map,
            // Self::U64Map(_) => ValueKind::U64Map,
            // Self::I64Map(_) => ValueKind::I64Map,
            // Self::StringMap(_) => ValueKind::StringMap,
            // Self::UuidMap(_) => ValueKind::UuidMap,
            // Self::U8Set(_) => ValueKind::U8Set,
            // Self::I8Set(_) => ValueKind::I8Set,
            // Self::U16Set(_) => ValueKind::U16Set,
            // Self::I16Set(_) => ValueKind::I16Set,
            // Self::U32Set(_) => ValueKind::U32Set,
            // Self::I32Set(_) => ValueKind::I32Set,
            // Self::U64Set(_) => ValueKind::U64Set,
            // Self::I64Set(_) => ValueKind::I64Set,
            // Self::StringSet(_) => ValueKind::StringSet,
            // Self::UuidSet(_) => ValueKind::UuidSet,
            // Self::Struct(_) => ValueKind::Struct,
            // Self::Enum(_) => ValueKind::Enum,
            Self::Sender(_) => ValueKind::Sender,
            Self::Receiver(_) => ValueKind::Receiver,
        }
    }
}

impl PrimaryTag for Value {
    type Tag = Self;
}

impl Serialize<Self> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(&self)
    }
}

impl Deserialize<Self> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|()| Self::None),

            ValueKind::Some => deserializer
                .deserialize_some::<Self, _>()
                .map(|v| Self::Some(Box::new(v))),

            ValueKind::Bool => deserializer.deserialize_bool().map(Self::Bool),
            ValueKind::U8 => deserializer.deserialize_u8().map(Self::U8),
            ValueKind::I8 => deserializer.deserialize_i8().map(Self::I8),
            ValueKind::U16 => deserializer.deserialize_u16().map(Self::U16),
            ValueKind::I16 => deserializer.deserialize_i16().map(Self::I16),
            ValueKind::U32 => deserializer.deserialize_u32().map(Self::U32),
            ValueKind::I32 => deserializer.deserialize_i32().map(Self::I32),
            ValueKind::U64 => deserializer.deserialize_u64().map(Self::U64),
            ValueKind::I64 => deserializer.deserialize_i64().map(Self::I64),
            ValueKind::F32 => deserializer.deserialize_f32().map(Self::F32),
            ValueKind::F64 => deserializer.deserialize_f64().map(Self::F64),
            ValueKind::String => deserializer.deserialize_string().map(Self::String),
            ValueKind::Uuid => deserializer.deserialize_uuid().map(Self::Uuid),
            ValueKind::ObjectId => deserializer.deserialize_object_id().map(Self::ObjectId),
            ValueKind::ServiceId => deserializer.deserialize_service_id().map(Self::ServiceId),

            ValueKind::Vec => deserializer
                .deserialize_vec_extend_new::<Self, _, _>()
                .map(Self::Vec),

            ValueKind::Sender => deserializer.deserialize_sender().map(Self::Sender),
            ValueKind::Receiver => deserializer.deserialize_receiver().map(Self::Receiver),
            _ => todo!(),
        }
    }
}

impl Serialize<Value> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Some(value) => serializer.serialize_some::<Value, _>(value),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::U8(value) => serializer.serialize_u8(*value),
            Value::I8(value) => serializer.serialize_i8(*value),
            Value::U16(value) => serializer.serialize_u16(*value),
            Value::I16(value) => serializer.serialize_i16(*value),
            Value::U32(value) => serializer.serialize_u32(*value),
            Value::I32(value) => serializer.serialize_i32(*value),
            Value::U64(value) => serializer.serialize_u64(*value),
            Value::I64(value) => serializer.serialize_i64(*value),
            Value::F32(value) => serializer.serialize_f32(*value),
            Value::F64(value) => serializer.serialize_f64(*value),
            Value::String(value) => serializer.serialize_string(value),
            Value::Uuid(value) => serializer.serialize_uuid(*value),
            Value::ObjectId(value) => serializer.serialize_object_id(*value),
            Value::ServiceId(value) => serializer.serialize_service_id(*value),
            Value::Vec(value) => serializer.serialize_vec_iter::<Value, _>(value),

            Value::Sender(value) => serializer.serialize_sender(*value),
            Value::Receiver(value) => serializer.serialize_receiver(*value),
        }
    }
}

impl Serialize<()> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<(), _>(&self)
    }
}

impl Deserialize<()> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<(), &Self>().map(|_| Self::None)
    }
}

impl Serialize<()> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::None => serializer.serialize_none(),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl Deserialize<()> for &Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|()| &Value::None),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl<T> Serialize<Option<T>> for Value
where
    T: Tag,
    Self: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::None => serializer.serialize_none(),
            Self::Some(value) => serializer.serialize_some::<T, _>(value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl<T> Deserialize<Option<T>> for Value
where
    T: Tag,
    Self: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.peek_value_kind()? {
            ValueKind::None => deserializer.deserialize_none().map(|()| Self::None),

            ValueKind::Some => deserializer
                .deserialize_some::<T, _>()
                .map(|v| Self::Some(Box::new(v))),

            _ => Err(DeserializeError::UnexpectedValue),
        }
    }
}

impl<'a, T> Serialize<Option<T>> for &'a Value
where
    T: Tag,
    &'a Value: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Some(value) => serializer.serialize_some::<T, _>(value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

macro_rules! impl_primitive {
    {
        $ty:ty, $kind:ident $( , $other:ty : $other_kind:ident )*
    } => {
        impl Serialize<$ty> for Value {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize::<$ty, &Self>(&self)
            }
        }

        impl Deserialize<$ty> for Value {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                match deserializer.peek_value_kind()? {
                    ValueKind::$kind => deserializer.deserialize::<$ty, $ty>().map(Self::$kind),
                    _ => Err(DeserializeError::UnexpectedValue),
                }
            }
        }

        impl Serialize<$ty> for &Value {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                match self {
                    Value::$kind(value) => serializer.serialize::<$ty, $ty>(*value),
                    $( Value::$other_kind(value) => serializer.serialize::<$ty, $other>(*value), )*
                    _ => Err(SerializeError::UnexpectedValue),
                }
            }
        }
    }
}

impl_primitive!(bool, Bool);
impl_primitive!(u8, U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64);
impl_primitive!(i8, I8, u8:U8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64);
impl_primitive!(u16, U16, u8:U8, i8:I8, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64);
impl_primitive!(i16, I16, u8:U8, i8:I8, u16:U16, u32:U32, i32:I32, u64:U64, i64:I64);
impl_primitive!(u32, U32, u8:U8, i8:I8, u16:U16, i16:I16, i32:I32, u64:U64, i64:I64);
impl_primitive!(i32, I32, u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, u64:U64, i64:I64);
impl_primitive!(u64, U64, u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, i64:I64);
impl_primitive!(i64, I64, u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64);
impl_primitive!(f32, F32, u8:U8, i8:I8, u16:U16, i16:I16);
impl_primitive!(f64, F64, u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, f32:F32);

impl Serialize<String> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<String, _>(&self)
    }
}

impl Deserialize<String> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_string().map(Self::String)
    }
}

impl Serialize<String> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::String(value) => serializer.serialize_string(value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl Serialize<Uuid> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Uuid, _>(&self)
    }
}

impl Deserialize<Uuid> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self::Uuid)
    }
}

impl Serialize<Uuid> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::Uuid(value) => serializer.serialize_uuid(*value),
            Value::Sender(value) => serializer.serialize_uuid(value.0),
            Value::Receiver(value) => serializer.serialize_uuid(value.0),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl Serialize<ObjectId> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ObjectId, _>(&self)
    }
}

impl Deserialize<ObjectId> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_object_id().map(Self::ObjectId)
    }
}

impl Serialize<ObjectId> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::ObjectId(value) => serializer.serialize_object_id(*value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl Serialize<ServiceId> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<ServiceId, _>(&self)
    }
}

impl Deserialize<ServiceId> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_service_id().map(Self::ServiceId)
    }
}

impl Serialize<ServiceId> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::ServiceId(value) => serializer.serialize_service_id(*value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl<T> Serialize<Vec<T>> for Value
where
    T: Tag,
    Self: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Vec(value) => serializer.serialize_vec_iter(value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl<T> Deserialize<Vec<T>> for Value
where
    T: Tag,
    Self: Deserialize<T>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_vec_extend_new().map(Self::Vec)
    }
}

impl<T> Serialize<Vec<T>> for &Value
where
    T: Tag,
    Self: Serialize<T>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::Vec(value) => serializer.serialize_vec_iter(value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

// TODO

impl<T: Tag> Serialize<Sender<T>> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Sender<T>, _>(&self)
    }
}

impl<T: Tag> Deserialize<Sender<T>> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(Self::Sender)
    }
}

impl<T: Tag> Serialize<Sender<T>> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::Uuid(value) => serializer.serialize_sender(ChannelCookie(*value)),
            Value::Sender(value) => serializer.serialize_sender(*value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

impl<T: Tag> Serialize<Receiver<T>> for Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Receiver<T>, _>(&self)
    }
}

impl<T: Tag> Deserialize<Receiver<T>> for Value {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(Self::Receiver)
    }
}

impl<T: Tag> Serialize<Receiver<T>> for &Value {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Value::Uuid(value) => serializer.serialize_receiver(ChannelCookie(*value)),
            Value::Receiver(value) => serializer.serialize_receiver(*value),
            _ => Err(SerializeError::UnexpectedValue),
        }
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for Value {
//     fn layout() -> Layout {
//         BuiltInType::Value.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::VALUE
//     }

//     fn add_references(_references: &mut References) {}
// }
