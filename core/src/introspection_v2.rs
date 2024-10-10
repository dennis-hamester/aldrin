mod built_in_type;
mod key_type;
mod layout;
mod lexical_id;
mod map_type;
mod result_type;

use crate::ids::TypeId;
use std::collections::btree_map::{BTreeMap, Entry};

pub use built_in_type::BuiltInType;
pub use key_type::KeyType;
pub use layout::Layout;
pub use lexical_id::LexicalId;
pub use map_type::MapType;
pub use result_type::ResultType;

#[derive(Debug, Clone)]
pub struct Introspection {
    type_id: TypeId,
    layout: Layout,
    type_ids: BTreeMap<LexicalId, TypeId>,
}

impl Introspection {
    pub fn builder<T: Introspectable>() -> IntrospectionBuilder {
        IntrospectionBuilder::new::<T>()
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug, Clone)]
pub struct IntrospectionBuilder {
    type_id: TypeId,
    layout: Layout,
    type_ids: BTreeMap<LexicalId, TypeId>,
}

impl IntrospectionBuilder {
    pub fn new<T: Introspectable>() -> Self {
        Self {
            type_id: T::type_id(),
            layout: T::layout(),
            type_ids: BTreeMap::new(),
        }
    }

    // pub fn add_type<T: Introspectable>(
    //     mut self,
    //     schema: impl AsRef<str>,
    //     name: impl Into<String>,
    // ) -> Self {
    //     let schema = schema.as_ref();

    //     let type_ids = match self.type_ids.get_mut(schema) {
    //         Some(type_ids) => type_ids,
    //         None => self.type_ids.entry(schema.to_owned()).or_default(),
    //     };

    //     match type_ids.entry(name.into()) {
    //         Entry::Vacant(entry) => {
    //             entry.insert(T::type_id());
    //         }

    //         Entry::Occupied(entry) => {
    //             if *entry.get() != T::type_id() {
    //                 panic!(
    //                     "duplicate type `{}::{}` added to introspection of `{}::{}`",
    //                     schema,
    //                     entry.key(),
    //                     self.schema,
    //                     self.layout.name(),
    //                 );
    //             }
    //         }
    //     }

    //     self
    // }

    pub fn finish(self) -> Introspection {
        Introspection {
            type_id: self.type_id,
            layout: self.layout,
            type_ids: self.type_ids,
        }
    }
}

pub trait Introspectable {
    fn type_id() -> TypeId;
    fn layout() -> Layout;
    fn lexical_id() -> LexicalId;
}
