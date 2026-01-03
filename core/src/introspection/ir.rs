mod array_type;
mod built_in_type;
mod enum_fallback;
mod enum_ty;
mod event;
mod event_fallback;
mod field;
mod function;
mod function_fallback;
mod layout;
mod map_type;
mod newtype;
mod result_type;
mod service;
mod struct_fallback;
mod struct_ty;
mod variant;

use super::{DynIntrospectable, Introspectable, LexicalId, References};
use crate::TypeId;
use std::collections::BTreeMap;

pub use array_type::ArrayTypeIr;
pub use built_in_type::BuiltInTypeIr;
pub use enum_fallback::{EnumFallbackIr, EnumFallbackIrBuilder};
pub use enum_ty::{EnumIr, EnumIrBuilder};
pub use event::{EventIr, EventIrBuilder};
pub use event_fallback::{EventFallbackIr, EventFallbackIrBuilder};
pub use field::{FieldIr, FieldIrBuilder};
pub use function::{FunctionIr, FunctionIrBuilder};
pub use function_fallback::{FunctionFallbackIr, FunctionFallbackIrBuilder};
pub use layout::LayoutIr;
pub use map_type::MapTypeIr;
pub use newtype::{NewtypeIr, NewtypeIrBuilder};
pub use result_type::ResultTypeIr;
pub use service::{ServiceIr, ServiceIrBuilder};
pub use struct_fallback::{StructFallbackIr, StructFallbackIrBuilder};
pub use struct_ty::{StructIr, StructIrBuilder};
pub use variant::{VariantIr, VariantIrBuilder};

#[derive(Debug, Clone)]
pub struct IntrospectionIr {
    pub(crate) type_id: TypeId,
    pub(crate) layout: LayoutIr,
    pub(crate) references: BTreeMap<LexicalId, TypeId>,
}

impl IntrospectionIr {
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

    pub fn layout(&self) -> &LayoutIr {
        &self.layout
    }

    pub fn references(&self) -> &BTreeMap<LexicalId, TypeId> {
        &self.references
    }

    pub fn resolve(&self, lexical_id: LexicalId) -> Option<TypeId> {
        self.references.get(&lexical_id).copied()
    }

    pub fn iter_references(&self) -> impl ExactSizeIterator<Item = (LexicalId, TypeId)> {
        self.references.iter().map(|(k, v)| (*k, *v))
    }

    pub fn as_built_in_layout(&self) -> Option<BuiltInTypeIr> {
        self.layout.as_built_in()
    }

    pub fn as_struct_layout(&self) -> Option<&StructIr> {
        self.layout.as_struct()
    }

    pub fn as_enum_layout(&self) -> Option<&EnumIr> {
        self.layout.as_enum()
    }

    pub fn as_service_layout(&self) -> Option<&ServiceIr> {
        self.layout.as_service()
    }

    pub fn as_newtype_layout(&self) -> Option<&NewtypeIr> {
        self.layout.as_newtype()
    }
}
