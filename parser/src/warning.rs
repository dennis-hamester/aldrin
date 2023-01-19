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
            Warning::DuplicateImport(w) => w.schema_name(),
            Warning::NonCamelCaseEnum(w) => w.schema_name(),
            Warning::NonCamelCaseEnumVariant(w) => w.schema_name(),
            Warning::NonCamelCaseService(w) => w.schema_name(),
            Warning::NonCamelCaseStruct(w) => w.schema_name(),
            Warning::NonShoutySnakeCaseConst(w) => w.schema_name(),
            Warning::NonSnakeCaseEvent(w) => w.schema_name(),
            Warning::NonSnakeCaseFunction(w) => w.schema_name(),
            Warning::NonSnakeCaseSchemaName(w) => w.schema_name(),
            Warning::NonSnakeCaseStructField(w) => w.schema_name(),
            Warning::UnusedImport(w) => w.schema_name(),
        }
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        match self {
            Warning::DuplicateImport(w) => w.format(parsed),
            Warning::NonCamelCaseEnum(w) => w.format(parsed),
            Warning::NonCamelCaseEnumVariant(w) => w.format(parsed),
            Warning::NonCamelCaseService(w) => w.format(parsed),
            Warning::NonCamelCaseStruct(w) => w.format(parsed),
            Warning::NonShoutySnakeCaseConst(w) => w.format(parsed),
            Warning::NonSnakeCaseEvent(w) => w.format(parsed),
            Warning::NonSnakeCaseFunction(w) => w.format(parsed),
            Warning::NonSnakeCaseSchemaName(w) => w.format(parsed),
            Warning::NonSnakeCaseStructField(w) => w.format(parsed),
            Warning::UnusedImport(w) => w.format(parsed),
        }
    }
}
