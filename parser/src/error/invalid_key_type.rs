use super::Error;
use crate::ast::{Definition, NamedRefKind, TypeName, TypeNameKind};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Schema};
use std::collections::HashSet;

#[derive(Debug)]
pub struct InvalidKeyType {
    schema_name: String,
    ty: TypeName,
    kind: InvalidKind,
}

impl InvalidKeyType {
    pub(crate) fn validate(ty: &TypeName, validate: &mut Validate) {
        if let Some(kind) = resolve_key_type(ty, validate.get_current_schema(), validate) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                ty: ty.clone(),
                kind,
            });
        }
    }

    pub fn type_name(&self) -> &TypeName {
        &self.ty
    }
}

#[derive(Debug)]
enum InvalidKind {
    BuiltIn,
    Struct,
    Enum,
    NewtypeToBuiltIn(String),
    NewtypeToStruct(String),
    NewtypeToEnum(String),
}

fn resolve_key_type<'a>(
    mut ty: &'a TypeName,
    main_schema: &'a Schema,
    validate: &'a Validate,
) -> Option<InvalidKind> {
    let mut schema = main_schema;
    let mut seen = HashSet::new();
    let mut is_newtype = false;

    loop {
        let named_ref = match ty.kind() {
            TypeNameKind::U8
            | TypeNameKind::I8
            | TypeNameKind::U16
            | TypeNameKind::I16
            | TypeNameKind::U32
            | TypeNameKind::I32
            | TypeNameKind::U64
            | TypeNameKind::I64
            | TypeNameKind::String
            | TypeNameKind::Uuid => break None,

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
                    break Some(InvalidKind::NewtypeToBuiltIn(ty.kind().to_string()));
                } else {
                    break Some(InvalidKind::BuiltIn);
                }
            }

            TypeNameKind::Ref(named_ref) => named_ref,
        };

        let (new_schema, ident) = match named_ref.kind() {
            NamedRefKind::Intern(ident) => (schema, ident),
            NamedRefKind::Extern(schema, ident) => (validate.get_schema(schema.value())?, ident),
        };

        if !seen.insert((new_schema.name(), ident.value())) {
            break None;
        }

        let def = new_schema
            .definitions()
            .iter()
            .find(|def| def.name().value() == ident.value())?;

        match def {
            Definition::Struct(_) => {
                if is_newtype {
                    let name = if new_schema.name() == main_schema.name() {
                        ident.value().to_owned()
                    } else {
                        format!("{}::{}", new_schema.name(), ident.value())
                    };

                    break Some(InvalidKind::NewtypeToStruct(name));
                } else {
                    break Some(InvalidKind::Struct);
                }
            }

            Definition::Enum(_) => {
                if is_newtype {
                    let name = if new_schema.name() == main_schema.name() {
                        ident.value().to_owned()
                    } else {
                        format!("{}::{}", new_schema.name(), ident.value())
                    };

                    break Some(InvalidKind::NewtypeToEnum(name));
                } else {
                    break Some(InvalidKind::Enum);
                }
            }

            Definition::Newtype(def) => {
                is_newtype = true;
                ty = def.target_type();
                schema = new_schema;
            }

            Definition::Service(_) | Definition::Const(_) => break None,
        }
    }
}

impl Diagnostic for InvalidKeyType {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let ty_kind = self.ty.kind();
        let mut fmt = Formatter::new(self, format!("invalid key type `{ty_kind}`"));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.ty.span().from,
                self.ty.span(),
                "type used here",
            );
        }

        fmt.help("allowed key types are `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,");
        fmt.help("`string`, `uuid` and newtypes resolving to one of those");

        match self.kind {
            InvalidKind::BuiltIn => {}

            InvalidKind::Struct => {
                fmt.note(format!("`{ty_kind}` is a struct"));
            }

            InvalidKind::Enum => {
                fmt.note(format!("`{ty_kind}` is an enum"));
            }

            InvalidKind::NewtypeToBuiltIn(ref name) => {
                fmt.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to `{name}`",
                ));
            }

            InvalidKind::NewtypeToStruct(ref name) => {
                fmt.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to the struct `{name}`",
                ));
            }

            InvalidKind::NewtypeToEnum(ref name) => {
                fmt.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to the enum `{name}`",
                ));
            }
        }

        fmt.format()
    }
}

impl From<InvalidKeyType> for Error {
    fn from(e: InvalidKeyType) -> Self {
        Self::InvalidKeyType(e)
    }
}
