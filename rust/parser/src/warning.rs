mod duplicate_import;
mod non_camel_case_enum;
mod non_camel_case_enum_variant;
mod non_camel_case_service;
mod non_camel_case_struct;
mod non_shouty_snake_case_const;
mod non_snake_case_function;
mod non_snake_case_schema_name;
mod non_snake_case_struct_field;
mod unused_import;

use crate::diag::{Diagnostic, DiagnosticKind};

pub use duplicate_import::DuplicateImport;
pub use non_camel_case_enum::NonCamelCaseEnum;
pub use non_camel_case_enum_variant::NonCamelCaseEnumVariant;
pub use non_camel_case_service::NonCamelCaseService;
pub use non_camel_case_struct::NonCamelCaseStruct;
pub use non_shouty_snake_case_const::NonShoutySnakeCaseConst;
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
            Warning::NonSnakeCaseFunction(w) => w.schema_name(),
            Warning::NonSnakeCaseSchemaName(w) => w.schema_name(),
            Warning::NonSnakeCaseStructField(w) => w.schema_name(),
            Warning::UnusedImport(w) => w.schema_name(),
        }
    }
}
