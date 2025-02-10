use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Value, ValueKind,
};

macro_rules! impl_primitive {
    {
        $tag:ty, $kind:ident, $ser:ident, $de:ident,
        $( :wide $( $wide:ty , )* )?
        $( :narrow $( $narrow:ty , )* )?
        $( :try $( $try:ty , )* )?
        $( :ser_conv $( $ser_conv:ty , )* )?
        $( :ser_try $( $ser_try:ty , )* )?
        $( :de_conv $( $de_conv:ty , )* )?
        $( :de_try $( $de_try:ty , )* )?
        $( :value $( $val_tag:ty : $val_kind:ident , )* )?
    } => {
        impl_primitive! {
            :done,
            $tag, $kind, $ser, $de,
            :ser_conv $( $( $ser_conv , )* )? $( $( $narrow , )* )?
            :ser_try $( $( $ser_try , )* )? $( $( $wide , )* )? $( $( $try , )* )?
            :de_conv $( $( $de_conv , )* )? $( $( $wide , )* )?
            :de_try $( $( $de_try , )* )? $( $( $narrow , )* )? $( $( $try , )* )?
            :value $( $( $val_tag : $val_kind , )* )?
        }
    };

    {
        :done,
        $tag:ty, $kind:ident, $ser:ident, $de:ident,
        :ser_conv $( $ser_conv:ty , )*
        :ser_try $( $ser_try:ty , )*
        :de_conv $( $de_conv:ty , )*
        :de_try $( $de_try:ty , )*
        :value $( $val_tag:ty : $val_kind:ident , )*
    } => {
        impl PrimaryTag for $tag {
            type Tag = Self;
        }

        impl Serialize<$tag> for $tag {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.$ser(self)
            }
        }

        impl Deserialize<$tag> for $tag {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                deserializer.$de()
            }
        }

        impl Serialize<$tag> for & $tag {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.$ser(*self)
            }
        }

        impl Serialize<Value> for $tag {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize::<$tag, $tag>(self)
            }
        }

        impl Deserialize<Value> for $tag {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                match deserializer.peek_value_kind()? {
                    ValueKind::$kind => deserializer.deserialize::<$tag, $tag>(),
                    $( ValueKind::$val_kind => deserializer.deserialize::<$val_tag, $tag>(), )*
                    _ => Err(DeserializeError::UnexpectedValue),
                }
            }
        }

        impl Serialize<Value> for & $tag {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize::<Value, $tag>(*self)
            }
        }

        $(
            impl Serialize<$tag> for $ser_conv {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    serializer.serialize::<$tag, $tag>(self.into())
                }
            }

            impl Serialize<$tag> for & $ser_conv {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    serializer.serialize::<$tag, $ser_conv>(*self)
                }
            }
        )*

        $(
            impl Serialize<$tag> for $ser_try {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    match self.try_into() {
                        Ok(value) => serializer.serialize::<$tag, $tag>(value),
                        Err(_) => Err(SerializeError::UnexpectedValue),
                    }
                }
            }

            impl Serialize<$tag> for & $ser_try {
                fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                    serializer.serialize::<$tag, $ser_try>(*self)
                }
            }
        )*

        $(
            impl Deserialize<$tag> for $de_conv {
                fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                    deserializer.deserialize::<$tag, $tag>().map(From::from)
                }
            }
        )*

        $(
            impl Deserialize<$tag> for $de_try {
                fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                    deserializer
                        .deserialize::<$tag, $tag>()?
                        .try_into()
                        .map_err(|_| DeserializeError::UnexpectedValue)
                }
            }
        )*
    };
}

impl_primitive! {
    bool, Bool, serialize_bool, deserialize_bool,
}

impl_primitive! {
    u8, U8, serialize_u8, deserialize_u8,
    :wide u16, i16, u32, i32, u64, i64, u128, i128, usize, isize,
    :try i8,
    :de_conv f32, f64,
    :value i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64,
}

impl_primitive! {
    i8, I8, serialize_i8, deserialize_i8,
    :wide i16, i32, i64, i128, isize,
    :try u8, u16, u32, u64, u128, usize,
    :de_conv f32, f64,
    :value u8:U8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64,
}

impl_primitive! {
    u16, U16, serialize_u16, deserialize_u16,
    :wide u32, i32, u64, i64, u128, i128, usize,
    :narrow u8,
    :try i8, i16, isize,
    :de_conv f32, f64,
    :value u8:U8, i8:I8, i16:I16, u32:U32, i32:I32, u64:U64, i64:I64,
}

impl_primitive! {
    i16, I16, serialize_i16, deserialize_i16,
    :wide i32, i64, i128, isize,
    :narrow u8, i8,
    :try u16, u32, u64, u128, usize,
    :de_conv f32, f64,
    :value u8:U8, i8:I8, u16:U16, u32:U32, i32:I32, u64:U64, i64:I64,
}

impl_primitive! {
    u32, U32, serialize_u32, deserialize_u32,
    :wide u64, i64, u128, i128,
    :narrow u8, u16,
    :try i8, i16, i32, usize, isize,
    :de_conv f64,
    :value u8:U8, i8:I8, u16:U16, i16:I16, i32:I32, u64:U64, i64:I64,
}

impl_primitive! {
    i32, I32, serialize_i32, deserialize_i32,
    :wide i64, i128,
    :narrow u8, i8, u16, i16,
    :try u32, u64, u128, usize, isize,
    :de_conv f64,
    :value u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, u64:U64, i64:I64,
}

impl_primitive! {
    u64, U64, serialize_u64, deserialize_u64,
    :wide u128,
    :narrow u8, u16, u32,
    :try i8, i16, i32, i64, i128, usize, isize,
    :value u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, i64:I64,
}

impl_primitive! {
    i64, I64, serialize_i64, deserialize_i64,
    :wide i128,
    :narrow u8, i8, u16, i16, u32, i32,
    :try u64, u128, usize, isize,
    :value u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, u64:U64,
}

impl_primitive! {
    f32, F32, serialize_f32, deserialize_f32,
    :ser_conv u8, i8, u16, i16,
    :de_try f64,
    :value u8:U8, i8:I8, u16:U16, i16:I16,
}

impl_primitive! {
    f64, F64, serialize_f64, deserialize_f64,
    :ser_conv u8, i8, u16, i16, u32, i32, f32,
    :value u8:U8, i8:I8, u16:U16, i16:I16, u32:U32, i32:I32, f32:F32,
}
