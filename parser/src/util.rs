use crate::ast::{ConstValue, Definition};
use crate::Schema;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

const THRESHOLD: f64 = 0.8;

// This should contain all builtin types, except for generics.
const BUILTIN_TYPES: &[&str] = &[
    "bool",
    "bytes",
    "f32",
    "f64",
    "i16",
    "i32",
    "i64",
    "i8",
    "lifetime",
    "object_id",
    "service_id",
    "string",
    "u16",
    "u32",
    "u64",
    "u8",
    "unit",
    "uuid",
    "value",
];

pub fn did_you_mean<'a, I>(candidates: I, value: &str) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    if let Some((candidate, score)) = candidates
        .into_iter()
        .map(|s| (s, strsim::jaro_winkler(s, value)))
        .max_by(|s1, s2| s1.1.partial_cmp(&s2.1).unwrap_or(Ordering::Equal))
    {
        if score > THRESHOLD {
            return Some(candidate);
        }
    }

    None
}

pub fn did_you_mean_type<'a>(
    schema: &'a Schema,
    name: &str,
    with_builtins: bool,
) -> Option<&'a str> {
    let candidates = schema.definitions().iter().filter_map(|d| match d {
        Definition::Struct(d) => Some(d.name().value()),
        Definition::Enum(d) => Some(d.name().value()),
        Definition::Service(_) | Definition::Const(_) => None,
    });

    if with_builtins {
        did_you_mean(candidates.chain(BUILTIN_TYPES.iter().copied()), name)
    } else {
        did_you_mean(candidates, name)
    }
}

pub fn did_you_mean_const_int<'a>(schema: &'a Schema, name: &str) -> Option<&'a str> {
    let candidates = schema
        .definitions()
        .iter()
        .filter_map(Definition::as_const)
        .filter(|const_def| {
            matches!(
                const_def.value(),
                ConstValue::U8(_)
                    | ConstValue::I8(_)
                    | ConstValue::U16(_)
                    | ConstValue::I16(_)
                    | ConstValue::U32(_)
                    | ConstValue::I32(_)
                    | ConstValue::U64(_)
                    | ConstValue::I64(_)
            )
        })
        .map(|d| d.name().value());

    did_you_mean(candidates, name)
}

pub fn find_duplicates<I, KFN, K, DFN>(iter: I, mut key_fn: KFN, mut dup_fn: DFN)
where
    I: IntoIterator,
    KFN: FnMut(&I::Item) -> K,
    K: Hash + Eq,
    DFN: FnMut(I::Item, &I::Item),
{
    let mut candidates: HashMap<_, Vec<_>> = HashMap::new();

    for elem in iter {
        candidates.entry(key_fn(&elem)).or_default().push(elem);
    }

    for (_, candidates) in candidates {
        if candidates.len() <= 1 {
            continue;
        }

        let mut duplicates = candidates.into_iter();
        let first = duplicates.next().unwrap();
        for duplicate in duplicates {
            dup_fn(duplicate, &first);
        }
    }
}
