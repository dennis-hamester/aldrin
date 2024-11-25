mod duplicate_definition;
mod duplicate_enum_variant;
mod duplicate_enum_variant_id;
mod duplicate_event_id;
mod duplicate_function_id;
mod duplicate_service_item;
mod duplicate_service_uuid;
mod duplicate_struct_field;
mod duplicate_struct_field_id;
mod empty_enum;
mod import_not_found;
mod invalid_const_value;
mod invalid_enum_variant_id;
mod invalid_event_id;
mod invalid_function_id;
mod invalid_schema_name;
mod invalid_service_uuid;
mod invalid_service_version;
mod invalid_struct_field_id;
mod invalid_syntax;
mod io_error;
mod keyword_as_ident;
mod missing_import;
mod recursive_type;
mod type_not_found;

use crate::diag::{Diagnostic, DiagnosticKind, Formatted};
use crate::Parsed;

pub use duplicate_definition::DuplicateDefinition;
pub use duplicate_enum_variant::DuplicateEnumVariant;
pub use duplicate_enum_variant_id::DuplicateEnumVariantId;
pub use duplicate_event_id::DuplicateEventId;
pub use duplicate_function_id::DuplicateFunctionId;
pub use duplicate_service_item::DuplicateServiceItem;
pub use duplicate_service_uuid::DuplicateServiceUuid;
pub use duplicate_struct_field::DuplicateStructField;
pub use duplicate_struct_field_id::DuplicateStructFieldId;
pub use empty_enum::EmptyEnum;
pub use import_not_found::ImportNotFound;
pub use invalid_const_value::InvalidConstValue;
pub use invalid_enum_variant_id::InvalidEnumVariantId;
pub use invalid_event_id::InvalidEventId;
pub use invalid_function_id::InvalidFunctionId;
pub use invalid_schema_name::InvalidSchemaName;
pub use invalid_service_uuid::InvalidServiceUuid;
pub use invalid_service_version::InvalidServiceVersion;
pub use invalid_struct_field_id::InvalidStructFieldId;
pub use invalid_syntax::{Expected, InvalidSyntax};
pub use io_error::IoError;
pub use keyword_as_ident::KeywordAsIdent;
pub use missing_import::MissingImport;
pub use recursive_type::{RecursiveEnum, RecursiveStruct};
pub use type_not_found::TypeNotFound;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    DuplicateDefinition(DuplicateDefinition),
    DuplicateEnumVariant(DuplicateEnumVariant),
    DuplicateEnumVariantId(DuplicateEnumVariantId),
    DuplicateEventId(DuplicateEventId),
    DuplicateFunctionId(DuplicateFunctionId),
    DuplicateServiceItem(DuplicateServiceItem),
    DuplicateServiceUuid(DuplicateServiceUuid),
    DuplicateStructField(DuplicateStructField),
    DuplicateStructFieldId(DuplicateStructFieldId),
    EmptyEnum(EmptyEnum),
    ImportNotFound(ImportNotFound),
    InvalidConstValue(InvalidConstValue),
    InvalidEnumVariantId(InvalidEnumVariantId),
    InvalidEventId(InvalidEventId),
    InvalidFunctionId(InvalidFunctionId),
    InvalidSchemaName(InvalidSchemaName),
    InvalidServiceUuid(InvalidServiceUuid),
    InvalidServiceVersion(InvalidServiceVersion),
    InvalidStructFieldId(InvalidStructFieldId),
    InvalidSyntax(InvalidSyntax),
    IoError(IoError),
    KeywordAsIdent(KeywordAsIdent),
    MissingImport(MissingImport),
    RecursiveEnum(RecursiveEnum),
    RecursiveStruct(RecursiveStruct),
    TypeNotFound(TypeNotFound),
}

impl Diagnostic for Error {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        match self {
            Self::DuplicateDefinition(e) => e.schema_name(),
            Self::DuplicateEnumVariant(e) => e.schema_name(),
            Self::DuplicateEnumVariantId(e) => e.schema_name(),
            Self::DuplicateEventId(e) => e.schema_name(),
            Self::DuplicateFunctionId(e) => e.schema_name(),
            Self::DuplicateServiceItem(e) => e.schema_name(),
            Self::DuplicateServiceUuid(e) => e.schema_name(),
            Self::DuplicateStructField(e) => e.schema_name(),
            Self::DuplicateStructFieldId(e) => e.schema_name(),
            Self::EmptyEnum(e) => e.schema_name(),
            Self::ImportNotFound(e) => e.schema_name(),
            Self::InvalidConstValue(e) => e.schema_name(),
            Self::InvalidEnumVariantId(e) => e.schema_name(),
            Self::InvalidEventId(e) => e.schema_name(),
            Self::InvalidFunctionId(e) => e.schema_name(),
            Self::InvalidSchemaName(e) => e.schema_name(),
            Self::InvalidServiceUuid(e) => e.schema_name(),
            Self::InvalidServiceVersion(e) => e.schema_name(),
            Self::InvalidStructFieldId(e) => e.schema_name(),
            Self::InvalidSyntax(e) => e.schema_name(),
            Self::IoError(e) => e.schema_name(),
            Self::KeywordAsIdent(e) => e.schema_name(),
            Self::MissingImport(e) => e.schema_name(),
            Self::RecursiveEnum(e) => e.schema_name(),
            Self::RecursiveStruct(e) => e.schema_name(),
            Self::TypeNotFound(e) => e.schema_name(),
        }
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        match self {
            Self::DuplicateDefinition(e) => e.format(parsed),
            Self::DuplicateEnumVariant(e) => e.format(parsed),
            Self::DuplicateEnumVariantId(e) => e.format(parsed),
            Self::DuplicateEventId(e) => e.format(parsed),
            Self::DuplicateFunctionId(e) => e.format(parsed),
            Self::DuplicateServiceItem(e) => e.format(parsed),
            Self::DuplicateServiceUuid(e) => e.format(parsed),
            Self::DuplicateStructField(e) => e.format(parsed),
            Self::DuplicateStructFieldId(e) => e.format(parsed),
            Self::EmptyEnum(e) => e.format(parsed),
            Self::ImportNotFound(e) => e.format(parsed),
            Self::InvalidConstValue(e) => e.format(parsed),
            Self::InvalidEnumVariantId(e) => e.format(parsed),
            Self::InvalidEventId(e) => e.format(parsed),
            Self::InvalidFunctionId(e) => e.format(parsed),
            Self::InvalidSchemaName(e) => e.format(parsed),
            Self::InvalidServiceUuid(e) => e.format(parsed),
            Self::InvalidServiceVersion(e) => e.format(parsed),
            Self::InvalidStructFieldId(e) => e.format(parsed),
            Self::InvalidSyntax(e) => e.format(parsed),
            Self::IoError(e) => e.format(parsed),
            Self::KeywordAsIdent(e) => e.format(parsed),
            Self::MissingImport(e) => e.format(parsed),
            Self::RecursiveEnum(e) => e.format(parsed),
            Self::RecursiveStruct(e) => e.format(parsed),
            Self::TypeNotFound(e) => e.format(parsed),
        }
    }
}
