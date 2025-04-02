use super::{DynIntrospectable, Introspectable, Layout, References, VERSION};
use crate::adapters::IterAsVec1;
use crate::tags::{self, PrimaryTag, Tag};
use crate::{Serialize, SerializeError, SerializedValue, Serializer, TypeId};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeSet;
use uuid::Uuid;

impl TypeId {
    pub fn compute<T: Introspectable + ?Sized>() -> Self {
        Self::compute_from_dyn(DynIntrospectable::new::<T>())
    }

    pub fn compute_from_dyn(ty: DynIntrospectable) -> Self {
        let mut compute = Compute::new(ty.layout());

        let mut references = Vec::new();
        ty.add_references(&mut References::new(&mut references));

        while let Some(ty) = references.pop() {
            if compute.add(ty.layout()) {
                ty.add_references(&mut References::new(&mut references));
            }
        }

        let serialized = SerializedValue::serialize(&compute).unwrap();
        Self(Uuid::new_v5(&compute.namespace(), &serialized))
    }
}

struct Compute {
    layout: Layout,
    referenced: BTreeSet<Layout>,
}

impl Compute {
    fn new(layout: Layout) -> Self {
        Self {
            layout,
            referenced: BTreeSet::new(),
        }
    }

    fn namespace(&self) -> Uuid {
        self.layout.namespace()
    }

    fn add(&mut self, layout: Layout) -> bool {
        self.referenced.insert(layout)
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ComputeField {
    Version = 0,
    Layout = 1,
    Referenced = 2,
}

impl Tag for Compute {}

impl PrimaryTag for Compute {
    type Tag = Self;
}

impl Serialize<Compute> for &Compute {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize::<tags::U32, _>(ComputeField::Version, VERSION)?;
        serializer.serialize::<Layout, _>(ComputeField::Layout, &self.layout)?;

        serializer.serialize::<tags::Vec<Layout>, _>(
            ComputeField::Referenced,
            IterAsVec1(&self.referenced),
        )?;

        serializer.finish()
    }
}
