mod built_in_type;
mod custom_type;
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
pub use custom_type::CustomType;
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
    type_ids: BTreeMap<String, BTreeMap<String, TypeId>>,
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

    pub fn inner_types(&self) -> &BTreeMap<String, BTreeMap<String, TypeId>> {
        &self.type_ids
    }

    pub fn iter_inner_types(&self) -> impl Iterator<Item = (&str, &str, TypeId)> + '_ {
        self.type_ids.iter().flat_map(|(schema, ids)| {
            ids.iter()
                .map(|(name, id)| (schema.as_str(), name.as_str(), *id))
        })
    }

    pub fn inner_type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.type_ids.values().flat_map(|ids| ids.values().copied())
    }

    pub fn resolve_schema(&self, schema: impl AsRef<str>) -> Option<&BTreeMap<String, TypeId>> {
        self.type_ids.get(schema.as_ref())
    }

    pub fn resolve_type(&self, schema: impl AsRef<str>, name: impl AsRef<str>) -> Option<TypeId> {
        self.resolve_schema(schema)?.get(name.as_ref()).copied()
    }

    pub fn resolve_custom_type(&self, custom_type: &CustomType) -> Option<TypeId> {
        self.resolve_type(custom_type.schema(), custom_type.name())
    }

    pub fn to_type_ref(&self) -> Option<TypeRef> {
        let name = match *self.layout {
            Layout::Struct(ref s) => s.name(),
            Layout::Enum(ref e) => e.name(),
            Layout::Service(_) => return None,
        };

        Some(TypeRef::custom(self.schema.as_ref(), name))
    }

    pub fn as_service_layout(&self) -> Option<&Service> {
        self.layout.as_service()
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
    type_ids: BTreeMap<String, BTreeMap<String, TypeId>>,
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

    pub fn add_type<T: Introspectable>(
        mut self,
        schema: impl AsRef<str>,
        name: impl Into<String>,
    ) -> Self {
        let schema = schema.as_ref();

        let type_ids = match self.type_ids.get_mut(schema) {
            Some(type_ids) => type_ids,
            None => self.type_ids.entry(schema.to_owned()).or_default(),
        };

        match type_ids.entry(name.into()) {
            Entry::Vacant(entry) => {
                entry.insert(T::type_id());
            }

            Entry::Occupied(entry) => {
                if *entry.get() != T::type_id() {
                    panic!(
                        "duplicate type `{}::{}` added to introspection of `{}::{}`",
                        schema,
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
