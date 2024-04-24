use super::{Introspectable, Layout, Types};
use crate::error::{DeserializeError, SerializeError};
use crate::serialized_value::SerializedValue;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub Uuid);

impl TypeId {
    /// Nil `TypeId` (all zeros).
    pub const NIL: Self = Self(Uuid::nil());

    /// Checks if the id is nil (all zeros).
    pub const fn is_nil(&self) -> bool {
        self.0.is_nil()
    }

    pub fn compute<T: Introspectable>() -> Self {
        let layout = T::layout();

        let mut types = Types::new();
        T::insert_types(&mut types);

        let compute = ComputeTypeId {
            schema: T::SCHEMA,
            layout,
            types: &types,
        };

        let serialized =
            SerializedValue::serialize(&compute).expect("failed to serialize introspection");

        Self(Uuid::new_v5(&layout.namespace(), &serialized))
    }
}

impl Serialize for TypeId {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0);
        Ok(())
    }
}

impl Deserialize for TypeId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

struct ComputeTypeId<'a> {
    schema: &'a str,
    layout: &'a Layout,
    types: &'a Types,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ComputeTypeIdField {
    Schema = 0,
    Layout = 1,
    Types = 2,
}

impl<'a> Serialize for ComputeTypeId<'a> {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(ComputeTypeIdField::Schema, self.schema)?;
        serializer.serialize_field(ComputeTypeIdField::Layout, self.layout)?;
        serializer.serialize_field(ComputeTypeIdField::Types, self.types)?;

        serializer.finish()
    }
}
