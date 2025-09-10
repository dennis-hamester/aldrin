use super::{Error, ErrorKind};
use crate::ast::TypeName;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::util::{self, InvalidKeyTypeKind};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub(crate) struct InvalidKeyType {
    schema_name: String,
    ty: TypeName,
    kind: InvalidKind,
}

impl InvalidKeyType {
    pub(crate) fn validate(ty: &TypeName, validate: &mut Validate) {
        let Err(kind) =
            util::resolves_to_key_type(ty.kind(), validate.get_current_schema(), validate)
        else {
            return;
        };

        let kind = match kind {
            InvalidKeyTypeKind::BuiltIn => InvalidKind::BuiltIn,
            InvalidKeyTypeKind::Struct => InvalidKind::Struct,
            InvalidKeyTypeKind::Enum => InvalidKind::Enum,

            InvalidKeyTypeKind::NewtypeToBuiltIn(kind) => {
                InvalidKind::NewtypeToBuiltIn(kind.to_string())
            }

            InvalidKeyTypeKind::NewtypeToStruct(schema, def) => {
                if schema.name() == validate.schema_name() {
                    InvalidKind::NewtypeToStruct(def.name().value().to_owned())
                } else {
                    InvalidKind::NewtypeToStruct(format!(
                        "{}::{}",
                        schema.name(),
                        def.name().value(),
                    ))
                }
            }

            InvalidKeyTypeKind::NewtypeToEnum(schema, def) => {
                if schema.name() == validate.schema_name() {
                    InvalidKind::NewtypeToEnum(def.name().value().to_owned())
                } else {
                    InvalidKind::NewtypeToEnum(
                        format!("{}::{}", schema.name(), def.name().value(),),
                    )
                }
            }

            InvalidKeyTypeKind::Other => return,
        };

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ty: ty.clone(),
            kind,
        });
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

impl Diagnostic for InvalidKeyType {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let ty_kind = self.ty.kind();
        let mut report = renderer.error(format!("invalid key type `{ty_kind}`"));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ty.span(), "type used here");
        }

        report = report.help(
            "allowed key types are `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,\n\
             `string`, `uuid` and newtypes resolving to one of those",
        );

        match self.kind {
            InvalidKind::BuiltIn => {}

            InvalidKind::Struct => {
                report = report.note(format!("`{ty_kind}` is a struct"));
            }

            InvalidKind::Enum => {
                report = report.note(format!("`{ty_kind}` is an enum"));
            }

            InvalidKind::NewtypeToBuiltIn(ref name) => {
                report = report.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to `{name}`",
                ));
            }

            InvalidKind::NewtypeToStruct(ref name) => {
                report = report.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to the struct `{name}`",
                ));
            }

            InvalidKind::NewtypeToEnum(ref name) => {
                report = report.note(format!(
                    "`{ty_kind}` is a newtype, that resolves to the enum `{name}`",
                ));
            }
        }

        report.render()
    }
}

impl From<InvalidKeyType> for Error {
    fn from(e: InvalidKeyType) -> Self {
        Self {
            kind: ErrorKind::InvalidKeyType(e),
        }
    }
}
