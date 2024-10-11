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
mod type_id;
mod variant;

use crate::error::{DeserializeError, SerializeError};
use crate::ids::TypeId;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::Uuid;

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
    type_ids: BTreeMap<LexicalId, TypeId>,
}

impl Introspection {
    pub fn new<T: Introspectable + ?Sized>() -> Self {
        Self::from_dyn(DynIntrospectable::new::<T>())
    }

    pub fn from_dyn(ty: DynIntrospectable) -> Self {
        let mut inner_types = Vec::new();
        ty.inner_types(&mut inner_types);

        let mut type_ids = BTreeMap::new();
        for ty in inner_types {
            let dup = type_ids.insert(ty.lexical_id(), TypeId::compute_from_dyn_v2(ty));
            assert_eq!(dup, None);
        }

        Self {
            type_id: TypeId::compute_from_dyn_v2(ty),
            layout: ty.layout(),
            type_ids,
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn type_ids(&self) -> &BTreeMap<LexicalId, TypeId> {
        &self.type_ids
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
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum IntrospectionField {
    Version = 0,
    TypeId = 1,
    Layout = 2,
    TypeIds = 3,
}

impl Serialize for Introspection {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        struct TypeIds<'a>(&'a BTreeMap<LexicalId, TypeId>);

        impl Serialize for TypeIds<'_> {
            fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
                serializer.serialize_map_iter(self.0.iter().map(|(k, v)| (k.0, v)))
            }
        }

        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize_field(IntrospectionField::Version, &VERSION)?;
        serializer.serialize_field(IntrospectionField::TypeId, &self.type_id)?;
        serializer.serialize_field(IntrospectionField::Layout, &self.layout)?;
        serializer.serialize_field(IntrospectionField::TypeIds, &TypeIds(&self.type_ids))?;

        serializer.finish()
    }
}

impl Deserialize for Introspection {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        #[derive(Default)]
        struct TypeIds(BTreeMap<LexicalId, TypeId>);

        impl Deserialize for TypeIds {
            fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
                deserializer.deserialize_map_extend_new()
            }
        }

        impl Extend<(Uuid, TypeId)> for TypeIds {
            fn extend<T>(&mut self, iter: T)
            where
                T: IntoIterator<Item = (Uuid, TypeId)>,
            {
                self.0
                    .extend(iter.into_iter().map(|(k, v)| (LexicalId(k), v)))
            }
        }

        let mut deserializer = deserializer.deserialize_struct()?;

        let version: u32 = deserializer.deserialize_specific_field(IntrospectionField::Version)?;
        if version != VERSION {
            return Err(DeserializeError::InvalidSerialization);
        }

        let type_id = deserializer.deserialize_specific_field(IntrospectionField::TypeId)?;
        let layout = deserializer.deserialize_specific_field(IntrospectionField::Layout)?;
        let type_ids: TypeIds =
            deserializer.deserialize_specific_field(IntrospectionField::TypeIds)?;

        deserializer.finish(Self {
            type_id,
            layout,
            type_ids: type_ids.0,
        })
    }
}

pub trait Introspectable {
    fn layout() -> Layout;
    fn lexical_id() -> LexicalId;
    fn inner_types(types: &mut Vec<DynIntrospectable>);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DynIntrospectable {
    layout: fn() -> Layout,
    lexical_id: fn() -> LexicalId,
    inner_types: fn(&mut Vec<DynIntrospectable>),
}

impl DynIntrospectable {
    pub fn new<T: Introspectable + ?Sized>() -> Self {
        Self {
            layout: T::layout,
            lexical_id: T::lexical_id,
            inner_types: T::inner_types,
        }
    }

    pub fn layout(self) -> Layout {
        (self.layout)()
    }

    pub fn lexical_id(self) -> LexicalId {
        (self.lexical_id)()
    }

    pub fn inner_types(self, types: &mut Vec<DynIntrospectable>) {
        (self.inner_types)(types)
    }
}
