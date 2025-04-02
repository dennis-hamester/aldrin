#[cfg(feature = "introspection")]
use crate::introspection::{
    DynIntrospectable, Introspectable, Layout, LexicalId, References, Struct,
};
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

macro_rules! impl_tuple {
    { $len:literal, $( ($tag:ident, $gen:ident, $idx:tt) ),+ } => {
        impl<$( $tag: Tag ),+> Tag for ($( $tag ,)+) { }

        impl<$( $tag: PrimaryTag ),+> PrimaryTag for ($( $tag ,)+) {
            type Tag = ($( $tag::Tag ,)+);
        }

        impl<$( $tag, $gen ),+> Serialize<($( $tag ,)+)> for ($( $gen ,)+)
        where
            $(
                $tag: Tag,
                $gen: Serialize<$tag>,
            )+
        {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                let mut serializer = serializer.serialize_struct2()?;

                $(
                    serializer.serialize($idx as u32, self.$idx)?;
                )+

                serializer.finish()
            }
        }

        impl<$( $tag, $gen ),+> Deserialize<($( $tag ,)+)> for ($( $gen ,)+)
        where
            $(
                $tag: Tag,
                $gen: Deserialize<$tag>,
            )+
        {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                let mut deserializer = deserializer.deserialize_struct()?;

                $(
                    #[allow(non_snake_case)]
                    let mut $gen = None;
                )+

                while let Some(deserializer) = deserializer.deserialize()? {
                    match deserializer.id() {
                        $( $idx => $gen = deserializer.deserialize().map(Some)?, )+
                        _ => return Err(DeserializeError::InvalidSerialization),
                    }
                }

                deserializer.finish_with(|_| {
                    Ok(($( $gen.ok_or(DeserializeError::InvalidSerialization)?, )+))
                })
            }
        }

        impl<'a, $( $tag, $gen ),+> Serialize<($( $tag ,)+)> for &'a ($( $gen ,)+)
        where
            $(
                $tag: Tag,
                &'a $gen: Serialize<$tag>,
            )+
        {
            fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
                let mut serializer = serializer.serialize_struct2()?;

                $(
                    serializer.serialize($idx as u32, &self.$idx)?;
                )+

                serializer.finish()
            }
        }

        #[cfg(feature = "introspection")]
        impl<$( $gen ),+> Introspectable for ($( $gen, )+)
        where
            $( $gen: Introspectable ),+
        {
            fn layout() -> Layout {
                Struct::builder("std", concat!("Tuple", $len))
                    $( .field($idx, concat!("field", $idx), true, $gen::lexical_id()) )+
                    .finish()
                    .into()
            }

            fn lexical_id() -> LexicalId {
                LexicalId::custom_generic(
                    "std",
                    concat!("Tuple", $len),
                    &[$( $gen::lexical_id() ),+],
                )
            }

            fn add_references(references: &mut References) {
                let types: [DynIntrospectable; $len] = [
                    $( DynIntrospectable::new::<$gen>() ),+
                ];

                references.extend(types);
            }
        }
    };
}

impl_tuple! { 1, (T0, U0, 0) }
impl_tuple! { 2, (T0, U0, 0), (T1, U1, 1) }
impl_tuple! { 3, (T0, U0, 0), (T1, U1, 1), (T2, U2, 2) }
impl_tuple! { 4, (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3) }
impl_tuple! { 5, (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4) }
impl_tuple! { 6, (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5) }

impl_tuple! {
    7,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6)
}

impl_tuple! {
    8,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6),
    (T7, U7, 7)
}

impl_tuple! {
    9,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6),
    (T7, U7, 7), (T8, U8, 8)
}

impl_tuple! {
    10,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6),
    (T7, U7, 7), (T8, U8, 8), (T9, U9, 9)
}

impl_tuple! {
    11,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6),
    (T7, U7, 7), (T8, U8, 8), (T9, U9, 9), (T10, U10, 10)
}

impl_tuple! {
    12,
    (T0, U0, 0), (T1, U1, 1), (T2, U2, 2), (T3, U3, 3), (T4, U4, 4), (T5, U5, 5), (T6, U6, 6),
    (T7, U7, 7), (T8, U8, 8), (T9, U9, 9), (T10, U10, 10), (T11, U11, 11)
}
