//! Private module
//!
//! This module and its contents are not part of the public API.

use super::{DynIntrospectable, Introspectable, LexicalId};

pub trait OptionHelper {
    fn lexical_id() -> LexicalId;
    fn dyn_introspectable() -> DynIntrospectable;
}

impl<T: Introspectable> OptionHelper for Option<T> {
    fn lexical_id() -> LexicalId {
        T::lexical_id()
    }

    fn dyn_introspectable() -> DynIntrospectable {
        DynIntrospectable::new::<T>()
    }
}
