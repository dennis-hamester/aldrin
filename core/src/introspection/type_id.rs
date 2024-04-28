use super::{Introspectable, Layout, Types};
use crate::error::SerializeError;
use crate::ids::TypeId;
use crate::serialized_value::SerializedValue;
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uuid::Uuid;

impl TypeId {
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
