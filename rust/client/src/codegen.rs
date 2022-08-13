//! Re-exports and utilities for the code generator.
//!
//! This module and its contents are not part of the public API.

pub use futures_core;
pub use uuid;

use aldrin_proto::{FromValue, Value};
use std::collections::HashMap;

pub fn get_struct_field<T: FromValue>(
    map: &mut HashMap<u32, Value>,
    idx: u32,
    dst: &mut Option<T>,
    required: bool,
) -> bool {
    match map.remove(&idx) {
        Some(field) => match field.convert() {
            Ok(field) => {
                *dst = Some(field);
                true
            }

            Err(e) => {
                if let Some(v) = e.0 {
                    map.insert(idx, v);
                }
                false
            }
        },

        None => !required,
    }
}
