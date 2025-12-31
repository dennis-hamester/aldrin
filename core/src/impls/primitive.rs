#[cfg(feature = "introspection")]
use crate::introspection::{Introspectable, LexicalId, References, ir};
use crate::tags::{self, PrimaryTag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

macro_rules! impl_primitive {
    {
        :tag $tag:ty ,
        :primary $( $primary:ty , )+
        :introspection $layout:ident, $lexid:ident,
        :ser_fn $ser_fn:ident ,
        :ser_for $( $ser_for:ty , )+
        :de_fn $de_fn:ident ,
        :de_for $( $de_for:ty , )+
    } => {
        $(
            impl PrimaryTag for $primary {
                type Tag = $tag;
            }

            #[cfg(feature = "introspection")]
            impl Introspectable for $primary {
                fn layout() -> ir::LayoutIr {
                    ir::BuiltInTypeIr::$layout.into()
                }

                fn lexical_id() -> LexicalId {
                    LexicalId::$lexid
                }

                fn add_references(_references: &mut References) {}
            }
        )+

        $(
            impl Serialize<$tag> for $ser_for {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    let val = self.try_into().map_err(|_| SerializeError::UnexpectedValue)?;
                    serializer.$ser_fn(val)
                }
            }

            impl Serialize<$tag> for & $ser_for {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    serializer.serialize::<$tag>(*self)
                }
            }
        )+

        $(
            impl Deserialize<$tag> for $de_for {
                fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                    deserializer.$de_fn()?.try_into().map_err(|_| DeserializeError::UnexpectedValue)
                }
            }
        )+
    };
}

impl_primitive! {
    :tag tags::Bool,
    :primary bool,
    :introspection Bool, BOOL,
    :ser_fn serialize_bool,
    :ser_for bool,
    :de_fn deserialize_bool,
    :de_for bool,
}

impl_primitive! {
    :tag tags::U8,
    :primary u8,
    :introspection U8, U8,
    :ser_fn serialize_u8,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_u8,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64,
}

impl_primitive! {
    :tag tags::I8,
    :primary i8,
    :introspection I8, I8,
    :ser_fn serialize_i8,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_i8,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64,
}

impl_primitive! {
    :tag tags::U16,
    :primary u16,
    :introspection U16, U16,
    :ser_fn serialize_u16,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_u16,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64,
}

impl_primitive! {
    :tag tags::I16,
    :primary i16,
    :introspection I16, I16,
    :ser_fn serialize_i16,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_i16,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64,
}

impl_primitive! {
    :tag tags::U32,
    :primary u32,
    :introspection U32, U32,
    :ser_fn serialize_u32,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_u32,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f64,
}

impl_primitive! {
    :tag tags::I32,
    :primary i32,
    :introspection I32, I32,
    :ser_fn serialize_i32,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_i32,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f64,
}

impl_primitive! {
    :tag tags::U64,
    :primary u64, usize,
    :introspection U64, U64,
    :ser_fn serialize_u64,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_u64,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
}

impl_primitive! {
    :tag tags::I64,
    :primary i64, isize,
    :introspection I64, I64,
    :ser_fn serialize_i64,
    :ser_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :de_fn deserialize_i64,
    :de_for u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
}

impl_primitive! {
    :tag tags::F32,
    :primary f32,
    :introspection F32, F32,
    :ser_fn serialize_f32,
    :ser_for u8, i8, u16, i16, f32,
    :de_fn deserialize_f32,
    :de_for f32, f64,
}

impl_primitive! {
    :tag tags::F64,
    :primary f64,
    :introspection F64, F64,
    :ser_fn serialize_f64,
    :ser_for u8, i8, u16, i16, u32, i32, f32, f64,
    :de_fn deserialize_f64,
    :de_for f64,
}
