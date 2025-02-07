mod array_type;
mod built_in_type;
mod enum_ty;
mod event;
mod field;
mod function;
mod key_type;
mod layout;
mod lexical_id;
mod map_type;
mod result_type;
mod service;
mod struct_ty;
#[cfg(test)]
mod test;
mod type_id;
mod variant;

#[doc(hidden)]
pub mod private;

use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

pub use array_type::ArrayType;
pub use built_in_type::BuiltInType;
pub use enum_ty::{Enum, EnumBuilder};
pub use event::Event;
pub use field::Field;
pub use function::Function;
pub use key_type::{KeyType, KeyTypeOf};
pub use layout::Layout;
pub use lexical_id::LexicalId;
pub use map_type::MapType;
pub use result_type::ResultType;
pub use service::{Service, ServiceBuilder};
pub use struct_ty::{Struct, StructBuilder};
pub use variant::Variant;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct Introspection {
    type_id: TypeId,
    layout: Layout,
    references: BTreeMap<LexicalId, TypeId>,
}

impl Introspection {
    pub fn new<T: Introspectable + ?Sized>() -> Self {
        Self::from_dyn(DynIntrospectable::new::<T>())
    }

    pub fn from_dyn(ty: DynIntrospectable) -> Self {
        let mut types = Vec::new();
        ty.add_references(&mut References::new(&mut types));

        let mut references = BTreeMap::new();
        for ty in types {
            let type_id = TypeId::compute_from_dyn(ty);
            let dup = references.insert(ty.lexical_id(), type_id);
            assert!(dup.is_none() || (dup == Some(type_id)));
        }

        Self {
            type_id: TypeId::compute_from_dyn(ty),
            layout: ty.layout(),
            references,
        }
    }

    pub fn lexical_id(&self) -> LexicalId {
        self.layout.lexical_id()
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn references(&self) -> &BTreeMap<LexicalId, TypeId> {
        &self.references
    }

    pub fn resolve(&self, lexical_id: LexicalId) -> Option<TypeId> {
        self.references.get(&lexical_id).copied()
    }

    pub fn iter_references(&self) -> impl ExactSizeIterator<Item = (LexicalId, TypeId)> + '_ {
        self.references.iter().map(|(k, v)| (*k, *v))
    }

    pub fn as_built_in_layout(&self) -> Option<BuiltInType> {
        self.layout.as_built_in()
    }

    pub fn as_struct_layout(&self) -> Option<&Struct> {
        self.layout.as_struct()
    }

    pub fn as_enum_layout(&self) -> Option<&Enum> {
        self.layout.as_enum()
    }

    pub fn as_service_layout(&self) -> Option<&Service> {
        self.layout.as_service()
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum IntrospectionField {
    Version = 0,
    TypeId = 1,
    Layout = 2,
    References = 3,
}

impl Tag for Introspection {}

impl PrimaryTag for Introspection {
    type Tag = Self;
}

impl Serialize<Self> for Introspection {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Introspection> for &Introspection {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize::<tags::U32, _>(IntrospectionField::Version, VERSION)?;
        serializer.serialize::<TypeId, _>(IntrospectionField::TypeId, self.type_id)?;
        serializer.serialize::<Layout, _>(IntrospectionField::Layout, &self.layout)?;

        serializer.serialize::<tags::Map<LexicalId, TypeId>, _>(
            IntrospectionField::References,
            &self.references,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Introspection {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut type_id = None;
        let mut layout = None;
        let mut references = None;

        while !deserializer.is_empty() {
            let deserializer = deserializer.deserialize()?;

            match deserializer.try_id() {
                Ok(IntrospectionField::Version) => {
                    if deserializer.deserialize::<tags::U32, u32>()? != VERSION {
                        return Err(DeserializeError::InvalidSerialization);
                    }
                }

                Ok(IntrospectionField::TypeId) => {
                    type_id = deserializer.deserialize::<TypeId, _>().map(Some)?;
                }

                Ok(IntrospectionField::Layout) => {
                    layout = deserializer.deserialize::<Layout, _>().map(Some)?;
                }

                Ok(IntrospectionField::References) => {
                    references = deserializer
                        .deserialize::<tags::Map<LexicalId, TypeId>, _>()
                        .map(Some)?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            type_id: type_id.ok_or(DeserializeError::InvalidSerialization)?,
            layout: layout.ok_or(DeserializeError::InvalidSerialization)?,
            references: references.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}

#[derive(Debug)]
pub struct References<'a> {
    inner: &'a mut Vec<DynIntrospectable>,
}

impl<'a> References<'a> {
    pub fn new(inner: &'a mut Vec<DynIntrospectable>) -> Self {
        Self { inner }
    }

    pub fn add<T: Introspectable + ?Sized>(&mut self) {
        self.add_dyn(DynIntrospectable::new::<T>());
    }

    pub fn add_dyn(&mut self, ty: DynIntrospectable) {
        self.inner.push(ty);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }
}

impl Extend<DynIntrospectable> for References<'_> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = DynIntrospectable>,
    {
        self.inner.extend(iter);
    }
}

pub trait Introspectable {
    fn layout() -> Layout;
    fn lexical_id() -> LexicalId;
    fn add_references(references: &mut References);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DynIntrospectable {
    layout: fn() -> Layout,
    lexical_id: fn() -> LexicalId,
    add_references: fn(&mut References),
}

impl DynIntrospectable {
    pub fn new<T: Introspectable + ?Sized>() -> Self {
        Self {
            layout: T::layout,
            lexical_id: T::lexical_id,
            add_references: T::add_references,
        }
    }

    pub fn layout(self) -> Layout {
        (self.layout)()
    }

    pub fn lexical_id(self) -> LexicalId {
        (self.lexical_id)()
    }

    pub fn add_references(self, references: &mut References) {
        (self.add_references)(references)
    }
}
