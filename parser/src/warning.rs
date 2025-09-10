mod duplicate_import;
mod non_camel_case_enum;
mod non_camel_case_enum_variant;
mod non_camel_case_newtype;
mod non_camel_case_service;
mod non_camel_case_struct;
mod non_shouty_snake_case_const;
mod non_snake_case_event;
mod non_snake_case_function;
mod non_snake_case_schema_name;
mod non_snake_case_struct_field;
mod reserved_ident;
mod reserved_schema_name;
mod unused_import;

use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::Parsed;

pub(crate) use duplicate_import::DuplicateImport;
pub(crate) use non_camel_case_enum::NonCamelCaseEnum;
pub(crate) use non_camel_case_enum_variant::NonCamelCaseEnumVariant;
pub(crate) use non_camel_case_newtype::NonCamelCaseNewtype;
pub(crate) use non_camel_case_service::NonCamelCaseService;
pub(crate) use non_camel_case_struct::NonCamelCaseStruct;
pub(crate) use non_shouty_snake_case_const::NonShoutySnakeCaseConst;
pub(crate) use non_snake_case_event::NonSnakeCaseEvent;
pub(crate) use non_snake_case_function::NonSnakeCaseFunction;
pub(crate) use non_snake_case_schema_name::NonSnakeCaseSchemaName;
pub(crate) use non_snake_case_struct_field::NonSnakeCaseStructField;
pub(crate) use reserved_ident::ReservedIdent;
pub(crate) use reserved_schema_name::ReservedSchemaName;
pub(crate) use unused_import::UnusedImport;

#[derive(Debug)]
pub struct Warning {
    kind: WarningKind,
}

impl Diagnostic for Warning {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        self.kind.schema_name()
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        self.kind.render(renderer, parsed)
    }
}

#[derive(Debug)]
enum WarningKind {
    DuplicateImport(DuplicateImport),
    NonCamelCaseEnum(NonCamelCaseEnum),
    NonCamelCaseEnumVariant(NonCamelCaseEnumVariant),
    NonCamelCaseNewtype(NonCamelCaseNewtype),
    NonCamelCaseService(NonCamelCaseService),
    NonCamelCaseStruct(NonCamelCaseStruct),
    NonShoutySnakeCaseConst(NonShoutySnakeCaseConst),
    NonSnakeCaseEvent(NonSnakeCaseEvent),
    NonSnakeCaseFunction(NonSnakeCaseFunction),
    NonSnakeCaseSchemaName(NonSnakeCaseSchemaName),
    NonSnakeCaseStructField(NonSnakeCaseStructField),
    ReservedIdent(ReservedIdent),
    ReservedSchemaName(ReservedSchemaName),
    UnusedImport(UnusedImport),
}

impl Diagnostic for WarningKind {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        match self {
            Self::DuplicateImport(w) => w.schema_name(),
            Self::NonCamelCaseEnum(w) => w.schema_name(),
            Self::NonCamelCaseEnumVariant(w) => w.schema_name(),
            Self::NonCamelCaseNewtype(w) => w.schema_name(),
            Self::NonCamelCaseService(w) => w.schema_name(),
            Self::NonCamelCaseStruct(w) => w.schema_name(),
            Self::NonShoutySnakeCaseConst(w) => w.schema_name(),
            Self::NonSnakeCaseEvent(w) => w.schema_name(),
            Self::NonSnakeCaseFunction(w) => w.schema_name(),
            Self::NonSnakeCaseSchemaName(w) => w.schema_name(),
            Self::NonSnakeCaseStructField(w) => w.schema_name(),
            Self::ReservedIdent(w) => w.schema_name(),
            Self::ReservedSchemaName(w) => w.schema_name(),
            Self::UnusedImport(w) => w.schema_name(),
        }
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        match self {
            Self::DuplicateImport(w) => w.render(renderer, parsed),
            Self::NonCamelCaseEnum(w) => w.render(renderer, parsed),
            Self::NonCamelCaseEnumVariant(w) => w.render(renderer, parsed),
            Self::NonCamelCaseNewtype(w) => w.render(renderer, parsed),
            Self::NonCamelCaseService(w) => w.render(renderer, parsed),
            Self::NonCamelCaseStruct(w) => w.render(renderer, parsed),
            Self::NonShoutySnakeCaseConst(w) => w.render(renderer, parsed),
            Self::NonSnakeCaseEvent(w) => w.render(renderer, parsed),
            Self::NonSnakeCaseFunction(w) => w.render(renderer, parsed),
            Self::NonSnakeCaseSchemaName(w) => w.render(renderer, parsed),
            Self::NonSnakeCaseStructField(w) => w.render(renderer, parsed),
            Self::ReservedIdent(w) => w.render(renderer, parsed),
            Self::ReservedSchemaName(w) => w.render(renderer, parsed),
            Self::UnusedImport(w) => w.render(renderer, parsed),
        }
    }
}
