mod built_in_type;
mod enum_ty;
mod event;
mod field;
mod function;
mod key_type;
mod layout;
mod map_type;
mod result_type;
mod service;
mod struct_ty;
mod type_id;
mod type_ref;
mod types;
mod variant;

use crate::error::{DeserializeError, SerializeError};
use crate::ids::TypeId;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, Entry};

pub use built_in_type::BuiltInType;
pub use enum_ty::{Enum, EnumBuilder};
pub use event::{Event, EventBuilder};
pub use field::Field;
pub use function::{Function, FunctionBuilder};
pub use key_type::KeyType;
pub use layout::Layout;
pub use map_type::MapType;
pub use result_type::ResultType;
pub use service::{Service, ServiceBuilder};
pub use struct_ty::{Struct, StructBuilder};
pub use type_ref::TypeRef;
pub use types::Types;
pub use variant::{Variant, VariantBuilder};

#[derive(Debug, Clone)]
pub struct Introspection {
    type_id: TypeId,
    schema: Cow<'static, str>,
    layout: Cow<'static, Layout>,
    type_ids: BTreeMap<String, TypeId>,
}

impl Introspection {
    pub fn builder<T: Introspectable>() -> IntrospectionBuilder {
        IntrospectionBuilder::new::<T>()
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        self.layout.name()
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn resolve_type(&self, name: impl AsRef<str>) -> Option<TypeId> {
        self.type_ids.get(name.as_ref()).copied()
    }

    pub fn to_type_ref(&self) -> Option<TypeRef> {
        let name = match *self.layout {
            Layout::Struct(ref s) => s.name(),
            Layout::Enum(ref e) => e.name(),
            Layout::Service(_) => return None,
        };

        Some(TypeRef::custom(format!("{}::{}", self.schema, name)))
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum IntrospectionField {
    TypeId = 0,
    Schema = 1,
    Layout = 2,
    TypeIds = 3,
}

impl Serialize for Introspection {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize_field(IntrospectionField::TypeId, &self.type_id)?;
        serializer.serialize_field(IntrospectionField::Schema, &self.schema)?;
        serializer.serialize_field(IntrospectionField::Layout, &self.layout)?;
        serializer.serialize_field(IntrospectionField::TypeIds, &self.type_ids)?;

        Ok(())
    }
}

impl Deserialize for Introspection {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_specific_field(IntrospectionField::TypeId)?;
        let schema = deserializer.deserialize_specific_field(IntrospectionField::Schema)?;
        let layout = deserializer.deserialize_specific_field(IntrospectionField::Layout)?;
        let type_ids = deserializer.deserialize_specific_field(IntrospectionField::TypeIds)?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            type_id,
            schema,
            layout,
            type_ids,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IntrospectionBuilder {
    type_id: TypeId,
    schema: &'static str,
    layout: &'static Layout,
    type_ids: BTreeMap<String, TypeId>,
}

impl IntrospectionBuilder {
    pub fn new<T: Introspectable>() -> Self {
        Self {
            type_id: T::type_id(),
            schema: T::SCHEMA,
            layout: T::layout(),
            type_ids: BTreeMap::new(),
        }
    }

    pub fn add_type<T: Introspectable>(mut self, name: impl Into<String>) -> Self {
        match self.type_ids.entry(name.into()) {
            Entry::Vacant(entry) => {
                entry.insert(T::type_id());
            }

            Entry::Occupied(entry) => {
                if *entry.get() != T::type_id() {
                    panic!(
                        "duplicate type `{}` added to introspection of `{}::{}`",
                        entry.key(),
                        self.schema,
                        self.layout.name(),
                    );
                }
            }
        }

        self
    }

    pub fn finish(self) -> Introspection {
        Introspection {
            type_id: self.type_id,
            schema: Cow::Borrowed(self.schema),
            layout: Cow::Borrowed(self.layout),
            type_ids: self.type_ids,
        }
    }
}

pub trait Introspectable {
    const SCHEMA: &'static str;

    fn introspection() -> &'static Introspection;
    fn layout() -> &'static Layout;
    fn insert_types(types: &mut Types);
    fn type_id() -> TypeId;
}
