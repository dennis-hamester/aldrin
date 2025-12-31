#[cfg(feature = "introspection")]
use crate::introspection::{Introspectable, LexicalId, References, ir};
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

impl<T: Tag, E: Tag> Tag for Result<T, E> {}

impl<T: PrimaryTag, E: PrimaryTag> PrimaryTag for Result<T, E> {
    type Tag = Result<T::Tag, E::Tag>;
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultVariant {
    Ok = 0,
    Err = 1,
}

impl<T, U, E, F> Serialize<Result<T, E>> for Result<U, F>
where
    T: Tag,
    U: Serialize<T>,
    E: Tag,
    F: Serialize<E>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Ok(value) => serializer.serialize_enum(ResultVariant::Ok, value),
            Err(value) => serializer.serialize_enum(ResultVariant::Err, value),
        }
    }
}

impl<'a, T, U, E, F> Serialize<Result<T, E>> for &'a Result<U, F>
where
    T: Tag,
    &'a U: Serialize<T>,
    E: Tag,
    &'a F: Serialize<E>,
{
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Ok(value) => serializer.serialize_enum(ResultVariant::Ok, value),
            Err(value) => serializer.serialize_enum(ResultVariant::Err, value),
        }
    }
}

impl<T, U, E, F> Deserialize<Result<T, E>> for Result<U, F>
where
    T: Tag,
    U: Deserialize<T>,
    E: Tag,
    F: Deserialize<E>,
{
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.try_id()? {
            ResultVariant::Ok => deserializer.deserialize().map(Ok),
            ResultVariant::Err => deserializer.deserialize().map(Err),
        }
    }
}

#[cfg(feature = "introspection")]
impl<T: Introspectable, E: Introspectable> Introspectable for Result<T, E> {
    fn layout() -> ir::LayoutIr {
        ir::BuiltInTypeIr::Result(ir::ResultTypeIr::new(T::lexical_id(), E::lexical_id())).into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::result(T::lexical_id(), E::lexical_id())
    }

    fn add_references(references: &mut References) {
        references.add::<T>();
        references.add::<E>();
    }
}
