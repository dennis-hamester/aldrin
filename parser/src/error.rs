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
mod extern_type_not_found;
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
pub use extern_type_not_found::ExternTypeNotFound;
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
    ExternTypeNotFound(ExternTypeNotFound),
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
            Error::DuplicateDefinition(e) => e.schema_name(),
            Error::DuplicateEnumVariant(e) => e.schema_name(),
            Error::DuplicateEnumVariantId(e) => e.schema_name(),
            Error::DuplicateEventId(e) => e.schema_name(),
            Error::DuplicateFunctionId(e) => e.schema_name(),
            Error::DuplicateServiceItem(e) => e.schema_name(),
            Error::DuplicateServiceUuid(e) => e.schema_name(),
            Error::DuplicateStructField(e) => e.schema_name(),
            Error::DuplicateStructFieldId(e) => e.schema_name(),
            Error::EmptyEnum(e) => e.schema_name(),
            Error::ExternTypeNotFound(e) => e.schema_name(),
            Error::ImportNotFound(e) => e.schema_name(),
            Error::InvalidConstValue(e) => e.schema_name(),
            Error::InvalidEnumVariantId(e) => e.schema_name(),
            Error::InvalidEventId(e) => e.schema_name(),
            Error::InvalidFunctionId(e) => e.schema_name(),
            Error::InvalidSchemaName(e) => e.schema_name(),
            Error::InvalidServiceUuid(e) => e.schema_name(),
            Error::InvalidServiceVersion(e) => e.schema_name(),
            Error::InvalidStructFieldId(e) => e.schema_name(),
            Error::InvalidSyntax(e) => e.schema_name(),
            Error::IoError(e) => e.schema_name(),
            Error::KeywordAsIdent(e) => e.schema_name(),
            Error::MissingImport(e) => e.schema_name(),
            Error::RecursiveEnum(e) => e.schema_name(),
            Error::RecursiveStruct(e) => e.schema_name(),
            Error::TypeNotFound(e) => e.schema_name(),
        }
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        match self {
            Error::DuplicateDefinition(e) => e.format(parsed),
            Error::DuplicateEnumVariant(e) => e.format(parsed),
            Error::DuplicateEnumVariantId(e) => e.format(parsed),
            Error::DuplicateEventId(e) => e.format(parsed),
            Error::DuplicateFunctionId(e) => e.format(parsed),
            Error::DuplicateServiceItem(e) => e.format(parsed),
            Error::DuplicateServiceUuid(e) => e.format(parsed),
            Error::DuplicateStructField(e) => e.format(parsed),
            Error::DuplicateStructFieldId(e) => e.format(parsed),
            Error::EmptyEnum(e) => e.format(parsed),
            Error::ExternTypeNotFound(e) => e.format(parsed),
            Error::ImportNotFound(e) => e.format(parsed),
            Error::InvalidConstValue(e) => e.format(parsed),
            Error::InvalidEnumVariantId(e) => e.format(parsed),
            Error::InvalidEventId(e) => e.format(parsed),
            Error::InvalidFunctionId(e) => e.format(parsed),
            Error::InvalidSchemaName(e) => e.format(parsed),
            Error::InvalidServiceUuid(e) => e.format(parsed),
            Error::InvalidServiceVersion(e) => e.format(parsed),
            Error::InvalidStructFieldId(e) => e.format(parsed),
            Error::InvalidSyntax(e) => e.format(parsed),
            Error::IoError(e) => e.format(parsed),
            Error::KeywordAsIdent(e) => e.format(parsed),
            Error::MissingImport(e) => e.format(parsed),
            Error::RecursiveEnum(e) => e.format(parsed),
            Error::RecursiveStruct(e) => e.format(parsed),
            Error::TypeNotFound(e) => e.format(parsed),
        }
    }
}
