use super::{Introspectable, Layout};
use crate::error::SerializeError;
use crate::value_serializer::{Serialize, Serializer};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct Types {
    types: Vec<&'static Layout>,
    set: HashSet<*const Layout>,
}

impl Types {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Introspectable>(&mut self) {
        let layout = T::layout();

        if self.set.insert(layout) {
            self.types.push(layout);
            T::insert_types(self);
        }
    }
}

impl Serialize for Types {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        self.types.serialize(serializer)
    }
}
