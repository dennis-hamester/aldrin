use super::{DynIntrospectable, Introspectable, Layout};
use crate::error::SerializeError;
use crate::ids::TypeId;
use crate::serialized_value::SerializedValue;
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeSet;
use uuid::Uuid;

impl TypeId {
    pub fn compute_v2<T: Introspectable + ?Sized>() -> Self {
        Self::compute_from_dyn_v2(DynIntrospectable::new::<T>())
    }

    pub fn compute_from_dyn_v2(ty: DynIntrospectable) -> Self {
        let mut compute = Compute::new(ty.layout());

        let mut inner_types = Vec::new();
        ty.inner_types(&mut inner_types);

        while let Some(ty) = inner_types.pop() {
            if compute.add(ty.layout()) {
                ty.inner_types(&mut inner_types);
            }
        }

        let serialized = SerializedValue::serialize(&compute).unwrap();
        Self(Uuid::new_v5(&compute.namespace(), &serialized))
    }
}

struct Compute {
    layout: Layout,
    inner_types: BTreeSet<Layout>,
}

impl Compute {
    fn new(layout: Layout) -> Self {
        Self {
            layout,
            inner_types: BTreeSet::new(),
        }
    }

    fn namespace(&self) -> Uuid {
        self.layout.namespace()
    }

    fn add(&mut self, layout: Layout) -> bool {
        self.inner_types.insert(layout)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ComputeField {
    Layout = 0,
    InnerTypes = 1,
}

impl Serialize for Compute {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        struct InnerTypes<'a>(&'a BTreeSet<Layout>);

        impl Serialize for InnerTypes<'_> {
            fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_vec_iter(self.0)
            }
        }

        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ComputeField::Layout, &self.layout)?;
        serializer.serialize_field(ComputeField::InnerTypes, &InnerTypes(&self.inner_types))?;

        serializer.finish()
    }
}
