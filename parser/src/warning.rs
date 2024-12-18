mod duplicate_import;
mod non_camel_case_enum;
mod non_camel_case_enum_variant;
mod non_camel_case_service;
mod non_camel_case_struct;
mod non_shouty_snake_case_const;
mod non_snake_case_event;
mod non_snake_case_function;
mod non_snake_case_schema_name;
mod non_snake_case_struct_field;
mod unused_import;

use crate::diag::{Diagnostic, DiagnosticKind, Formatted};
use crate::Parsed;

pub use duplicate_import::DuplicateImport;
pub use non_camel_case_enum::NonCamelCaseEnum;
pub use non_camel_case_enum_variant::NonCamelCaseEnumVariant;
pub use non_camel_case_service::NonCamelCaseService;
pub use non_camel_case_struct::NonCamelCaseStruct;
pub use non_shouty_snake_case_const::NonShoutySnakeCaseConst;
pub use non_snake_case_event::NonSnakeCaseEvent;
pub use non_snake_case_function::NonSnakeCaseFunction;
pub use non_snake_case_schema_name::NonSnakeCaseSchemaName;
pub use non_snake_case_struct_field::NonSnakeCaseStructField;
pub use unused_import::UnusedImport;

#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    DuplicateImport(DuplicateImport),
    NonCamelCaseEnum(NonCamelCaseEnum),
    NonCamelCaseEnumVariant(NonCamelCaseEnumVariant),
    NonCamelCaseService(NonCamelCaseService),
    NonCamelCaseStruct(NonCamelCaseStruct),
    NonShoutySnakeCaseConst(NonShoutySnakeCaseConst),
    NonSnakeCaseEvent(NonSnakeCaseEvent),
    NonSnakeCaseFunction(NonSnakeCaseFunction),
    NonSnakeCaseSchemaName(NonSnakeCaseSchemaName),
    NonSnakeCaseStructField(NonSnakeCaseStructField),
    UnusedImport(UnusedImport),
}

impl Diagnostic for Warning {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        match self {
            Self::DuplicateImport(w) => w.schema_name(),
            Self::NonCamelCaseEnum(w) => w.schema_name(),
            Self::NonCamelCaseEnumVariant(w) => w.schema_name(),
            Self::NonCamelCaseService(w) => w.schema_name(),
            Self::NonCamelCaseStruct(w) => w.schema_name(),
            Self::NonShoutySnakeCaseConst(w) => w.schema_name(),
            Self::NonSnakeCaseEvent(w) => w.schema_name(),
            Self::NonSnakeCaseFunction(w) => w.schema_name(),
            Self::NonSnakeCaseSchemaName(w) => w.schema_name(),
            Self::NonSnakeCaseStructField(w) => w.schema_name(),
            Self::UnusedImport(w) => w.schema_name(),
        }
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        match self {
            Self::DuplicateImport(w) => w.format(parsed),
            Self::NonCamelCaseEnum(w) => w.format(parsed),
            Self::NonCamelCaseEnumVariant(w) => w.format(parsed),
            Self::NonCamelCaseService(w) => w.format(parsed),
            Self::NonCamelCaseStruct(w) => w.format(parsed),
            Self::NonShoutySnakeCaseConst(w) => w.format(parsed),
            Self::NonSnakeCaseEvent(w) => w.format(parsed),
            Self::NonSnakeCaseFunction(w) => w.format(parsed),
            Self::NonSnakeCaseSchemaName(w) => w.format(parsed),
            Self::NonSnakeCaseStructField(w) => w.format(parsed),
            Self::UnusedImport(w) => w.format(parsed),
        }
    }
}
