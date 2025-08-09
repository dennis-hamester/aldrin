use crate::ast::{
    ConstValue, Definition, EnumDef, NamedRef, NamedRefKind, StructDef, TypeNameKind,
};
use crate::validate::Validate;
use crate::Schema;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use Language::Rust;
use ReservedKind::{Builtin, Keyword};

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

const BUILTIN_KEY_TYPES: &[&str] = &[
    "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "string", "uuid",
];

const RESERVED_NAMES: &[(&str, ReservedUsage)] = &[
    ("Self", &[(Keyword, &[Rust])]),
    ("as", &[(Keyword, &[Rust])]),
    ("async", &[(Keyword, &[Rust])]),
    ("await", &[(Keyword, &[Rust])]),
    ("bool", &[(Builtin, &[Rust])]),
    ("break", &[(Keyword, &[Rust])]),
    ("char", &[(Builtin, &[Rust])]),
    ("const", &[(Keyword, &[Rust])]),
    ("continue", &[(Keyword, &[Rust])]),
    ("crate", &[(Keyword, &[Rust])]),
    ("dyn", &[(Keyword, &[Rust])]),
    ("else", &[(Keyword, &[Rust])]),
    ("enum", &[(Keyword, &[Rust])]),
    ("extern", &[(Keyword, &[Rust])]),
    ("f128", &[(Builtin, &[Rust])]),
    ("f16", &[(Builtin, &[Rust])]),
    ("f32", &[(Builtin, &[Rust])]),
    ("f64", &[(Builtin, &[Rust])]),
    ("false", &[(Keyword, &[Rust])]),
    ("fn", &[(Keyword, &[Rust])]),
    ("for", &[(Keyword, &[Rust])]),
    ("i128", &[(Builtin, &[Rust])]),
    ("i16", &[(Builtin, &[Rust])]),
    ("i32", &[(Builtin, &[Rust])]),
    ("i64", &[(Builtin, &[Rust])]),
    ("i8", &[(Builtin, &[Rust])]),
    ("if", &[(Keyword, &[Rust])]),
    ("impl", &[(Keyword, &[Rust])]),
    ("in", &[(Keyword, &[Rust])]),
    ("isize", &[(Builtin, &[Rust])]),
    ("let", &[(Keyword, &[Rust])]),
    ("loop", &[(Keyword, &[Rust])]),
    ("match", &[(Keyword, &[Rust])]),
    ("mod", &[(Keyword, &[Rust])]),
    ("move", &[(Keyword, &[Rust])]),
    ("mut", &[(Keyword, &[Rust])]),
    ("pub", &[(Keyword, &[Rust])]),
    ("ref", &[(Keyword, &[Rust])]),
    ("return", &[(Keyword, &[Rust])]),
    ("self", &[(Keyword, &[Rust])]),
    ("static", &[(Keyword, &[Rust])]),
    ("str", &[(Builtin, &[Rust])]),
    ("struct", &[(Keyword, &[Rust])]),
    ("super", &[(Keyword, &[Rust])]),
    ("trait", &[(Keyword, &[Rust])]),
    ("true", &[(Keyword, &[Rust])]),
    ("type", &[(Keyword, &[Rust])]),
    ("u128", &[(Builtin, &[Rust])]),
    ("u16", &[(Builtin, &[Rust])]),
    ("u32", &[(Builtin, &[Rust])]),
    ("u64", &[(Builtin, &[Rust])]),
    ("u8", &[(Builtin, &[Rust])]),
    ("union", &[(Keyword, &[Rust])]),
    ("unsafe", &[(Keyword, &[Rust])]),
    ("use", &[(Keyword, &[Rust])]),
    ("usize", &[(Builtin, &[Rust])]),
    ("where", &[(Keyword, &[Rust])]),
    ("while", &[(Keyword, &[Rust])]),
];

pub(crate) type ReservedUsage = &'static [(ReservedKind, &'static [Language])];

pub(crate) fn did_you_mean<'a, I>(candidates: I, value: &str) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    candidates
        .into_iter()
        .map(|s| (s, strsim::jaro_winkler(s, value)))
        .max_by(|s1, s2| s1.1.partial_cmp(&s2.1).unwrap_or(Ordering::Equal))
        .and_then(|(candidate, score)| (score > THRESHOLD).then_some(candidate))
}

pub(crate) fn did_you_mean_type<'a>(
    schema: &'a Schema,
    name: &str,
    with_builtins: bool,
) -> Option<&'a str> {
    let candidates = schema.definitions().iter().filter_map(|d| match d {
        Definition::Struct(d) => Some(d.name().value()),
        Definition::Enum(d) => Some(d.name().value()),
        Definition::Newtype(d) => Some(d.name().value()),
        Definition::Service(_) | Definition::Const(_) => None,
    });

    if with_builtins {
        did_you_mean(candidates.chain(BUILTIN_TYPES.iter().copied()), name)
    } else {
        did_you_mean(candidates, name)
    }
}

pub(crate) fn did_you_mean_key_type<'a>(
    schema: &'a Schema,
    name: &str,
    with_builtins: bool,
    validate: &Validate,
) -> Option<&'a str> {
    let candidates = schema
        .definitions()
        .iter()
        .filter_map(Definition::as_newtype)
        .filter_map(|newtype| {
            let kind = TypeNameKind::Ref(NamedRef::dummy_intern(newtype.name().clone()));

            resolves_to_key_type(&kind, schema, validate)
                .ok()
                .map(|_| newtype.name().value())
        });

    if with_builtins {
        did_you_mean(candidates.chain(BUILTIN_KEY_TYPES.iter().copied()), name)
    } else {
        did_you_mean(candidates, name)
    }
}

pub(crate) fn did_you_mean_const_int<'a>(schema: &'a Schema, name: &str) -> Option<&'a str> {
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

pub(crate) fn find_duplicates<I, KFN, K, DFN>(iter: I, mut key_fn: KFN, mut dup_fn: DFN)
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

pub(crate) fn resolves_to_key_type<'a>(
    mut kind: &'a TypeNameKind,
    mut schema: &'a Schema,
    validate: &'a Validate,
) -> Result<(), InvalidKeyTypeKind<'a>> {
    let mut seen = HashSet::new();
    let mut is_newtype = false;

    loop {
        let named_ref = match kind {
            TypeNameKind::U8
            | TypeNameKind::I8
            | TypeNameKind::U16
            | TypeNameKind::I16
            | TypeNameKind::U32
            | TypeNameKind::I32
            | TypeNameKind::U64
            | TypeNameKind::I64
            | TypeNameKind::String
            | TypeNameKind::Uuid => break Ok(()),

            TypeNameKind::Bool
            | TypeNameKind::F32
            | TypeNameKind::F64
            | TypeNameKind::ObjectId
            | TypeNameKind::ServiceId
            | TypeNameKind::Value
            | TypeNameKind::Option(_)
            | TypeNameKind::Box(_)
            | TypeNameKind::Vec(_)
            | TypeNameKind::Bytes
            | TypeNameKind::Map(_, _)
            | TypeNameKind::Set(_)
            | TypeNameKind::Sender(_)
            | TypeNameKind::Receiver(_)
            | TypeNameKind::Lifetime
            | TypeNameKind::Unit
            | TypeNameKind::Result(_, _)
            | TypeNameKind::Array(_, _) => {
                if is_newtype {
                    break Err(InvalidKeyTypeKind::NewtypeToBuiltIn(kind));
                } else {
                    break Err(InvalidKeyTypeKind::BuiltIn);
                }
            }

            TypeNameKind::Ref(named_ref) => named_ref,
        };

        let (new_schema, ident) = match named_ref.kind() {
            NamedRefKind::Intern(ident) => (schema, ident),

            NamedRefKind::Extern(schema, ident) => (
                validate
                    .get_schema(schema.value())
                    .ok_or(InvalidKeyTypeKind::Other)?,
                ident,
            ),
        };

        if !seen.insert((new_schema.name(), ident.value())) {
            break Err(InvalidKeyTypeKind::Other);
        }

        let def = new_schema
            .definitions()
            .iter()
            .find(|def| def.name().value() == ident.value())
            .ok_or(InvalidKeyTypeKind::Other)?;

        match def {
            Definition::Struct(def) => {
                if is_newtype {
                    break Err(InvalidKeyTypeKind::NewtypeToStruct(new_schema, def));
                } else {
                    break Err(InvalidKeyTypeKind::Struct);
                }
            }

            Definition::Enum(def) => {
                if is_newtype {
                    break Err(InvalidKeyTypeKind::NewtypeToEnum(new_schema, def));
                } else {
                    break Err(InvalidKeyTypeKind::Enum);
                }
            }

            Definition::Newtype(def) => {
                is_newtype = true;
                kind = def.target_type().kind();
                schema = new_schema;
            }

            Definition::Service(_) | Definition::Const(_) => break Err(InvalidKeyTypeKind::Other),
        }
    }
}

#[derive(Debug)]
pub(crate) enum InvalidKeyTypeKind<'a> {
    BuiltIn,
    Struct,
    Enum,
    NewtypeToBuiltIn(&'a TypeNameKind),
    NewtypeToStruct(&'a Schema, &'a StructDef),
    NewtypeToEnum(&'a Schema, &'a EnumDef),
    Other,
}

pub(crate) fn is_reserved_name(name: &str) -> Option<ReservedUsage> {
    RESERVED_NAMES
        .binary_search_by_key(&name, |reserved| reserved.0)
        .ok()
        .map(|idx| RESERVED_NAMES[idx].1)
}

#[derive(Debug)]
pub(crate) enum Language {
    Rust,
}

impl Language {
    pub(crate) fn fmt_list(langs: &'static [Self]) -> impl fmt::Display {
        struct FmtList(&'static [Language]);

        impl fmt::Display for FmtList {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for (i, lang) in self.0.iter().enumerate() {
                    if i > 0 {
                        if i < (self.0.len() - 1) {
                            write!(f, ", ")?;
                        } else {
                            write!(f, " and ")?;
                        }
                    }

                    lang.fmt(f)?;
                }

                Ok(())
            }
        }

        FmtList(langs)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ReservedKind {
    Builtin,
    Keyword,
}

impl fmt::Display for ReservedKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Builtin => write!(f, "a built-in type"),
            Self::Keyword => write!(f, "a keyword"),
        }
    }
}

#[cfg(test)]
#[test]
fn reserved_names_are_sorted_and_unique() {
    let mut last = RESERVED_NAMES[0].0;
    for reserved in RESERVED_NAMES.iter().skip(1).map(|reserved| reserved.0) {
        assert!(reserved > last);
        last = reserved;
    }
}
