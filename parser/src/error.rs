mod const_int_not_found;
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
mod expected_const_int_found_service;
mod expected_const_int_found_string;
mod expected_const_int_found_type;
mod expected_const_int_found_uuid;
mod expected_type_found_const;
mod expected_type_found_service;
mod import_not_found;
mod invalid_array_len;
mod invalid_const_value;
mod invalid_enum_variant_id;
mod invalid_escape_code;
mod invalid_event_id;
mod invalid_function_id;
mod invalid_ident;
mod invalid_key_type;
mod invalid_schema_name;
mod invalid_service_version;
mod invalid_struct_field_id;
mod invalid_syntax;
mod io_error;
mod missing_import;
mod recursive_type;
mod type_not_found;

use crate::Parser;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};

pub(crate) use const_int_not_found::ConstIntNotFound;
pub(crate) use duplicate_definition::DuplicateDefinition;
pub(crate) use duplicate_enum_variant::DuplicateEnumVariant;
pub(crate) use duplicate_enum_variant_id::DuplicateEnumVariantId;
pub(crate) use duplicate_event_id::DuplicateEventId;
pub(crate) use duplicate_function_id::DuplicateFunctionId;
pub(crate) use duplicate_service_item::DuplicateServiceItem;
pub(crate) use duplicate_service_uuid::DuplicateServiceUuid;
pub(crate) use duplicate_struct_field::DuplicateStructField;
pub(crate) use duplicate_struct_field_id::DuplicateStructFieldId;
pub(crate) use empty_enum::EmptyEnum;
pub(crate) use expected_const_int_found_service::ExpectedConstIntFoundService;
pub(crate) use expected_const_int_found_string::ExpectedConstIntFoundString;
pub(crate) use expected_const_int_found_type::ExpectedConstIntFoundType;
pub(crate) use expected_const_int_found_uuid::ExpectedConstIntFoundUuid;
pub(crate) use expected_type_found_const::ExpectedTypeFoundConst;
pub(crate) use expected_type_found_service::ExpectedTypeFoundService;
pub(crate) use import_not_found::ImportNotFound;
pub(crate) use invalid_array_len::InvalidArrayLen;
pub(crate) use invalid_const_value::InvalidConstValue;
pub(crate) use invalid_enum_variant_id::InvalidEnumVariantId;
pub(crate) use invalid_escape_code::InvalidEscapeCode;
pub(crate) use invalid_event_id::InvalidEventId;
pub(crate) use invalid_function_id::InvalidFunctionId;
pub(crate) use invalid_ident::InvalidIdent;
pub(crate) use invalid_key_type::InvalidKeyType;
pub(crate) use invalid_schema_name::InvalidSchemaName;
pub(crate) use invalid_service_version::InvalidServiceVersion;
pub(crate) use invalid_struct_field_id::InvalidStructFieldId;
pub(crate) use invalid_syntax::InvalidSyntax;
pub(crate) use io_error::IoError;
pub(crate) use missing_import::MissingImport;
pub(crate) use recursive_type::{RecursiveEnum, RecursiveNewtype, RecursiveStruct};
pub(crate) use type_not_found::TypeNotFound;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn error_kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Diagnostic for Error {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        self.kind.schema_name()
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        self.kind.render(renderer, parser)
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    ConstIntNotFound(ConstIntNotFound),
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
    ExpectedConstIntFoundService(ExpectedConstIntFoundService),
    ExpectedConstIntFoundString(ExpectedConstIntFoundString),
    ExpectedConstIntFoundType(ExpectedConstIntFoundType),
    ExpectedConstIntFoundUuid(ExpectedConstIntFoundUuid),
    ExpectedTypeFoundConst(ExpectedTypeFoundConst),
    ExpectedTypeFoundService(ExpectedTypeFoundService),
    ImportNotFound(ImportNotFound),
    InvalidArrayLen(InvalidArrayLen),
    InvalidConstValue(InvalidConstValue),
    InvalidEnumVariantId(InvalidEnumVariantId),
    InvalidEscapeCode(InvalidEscapeCode),
    InvalidEventId(InvalidEventId),
    InvalidFunctionId(InvalidFunctionId),
    InvalidIdent(InvalidIdent),
    InvalidKeyType(InvalidKeyType),
    InvalidSchemaName(InvalidSchemaName),
    InvalidServiceVersion(InvalidServiceVersion),
    InvalidStructFieldId(InvalidStructFieldId),
    InvalidSyntax(InvalidSyntax),
    IoError(IoError),
    MissingImport(MissingImport),
    RecursiveEnum(RecursiveEnum),
    RecursiveNewtype(RecursiveNewtype),
    RecursiveStruct(RecursiveStruct),
    TypeNotFound(TypeNotFound),
}

impl Diagnostic for ErrorKind {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        match self {
            Self::ConstIntNotFound(e) => e.schema_name(),
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
            Self::ExpectedConstIntFoundService(e) => e.schema_name(),
            Self::ExpectedConstIntFoundString(e) => e.schema_name(),
            Self::ExpectedConstIntFoundType(e) => e.schema_name(),
            Self::ExpectedConstIntFoundUuid(e) => e.schema_name(),
            Self::ExpectedTypeFoundConst(e) => e.schema_name(),
            Self::ExpectedTypeFoundService(e) => e.schema_name(),
            Self::ImportNotFound(e) => e.schema_name(),
            Self::InvalidArrayLen(e) => e.schema_name(),
            Self::InvalidConstValue(e) => e.schema_name(),
            Self::InvalidEnumVariantId(e) => e.schema_name(),
            Self::InvalidEscapeCode(e) => e.schema_name(),
            Self::InvalidEventId(e) => e.schema_name(),
            Self::InvalidFunctionId(e) => e.schema_name(),
            Self::InvalidIdent(e) => e.schema_name(),
            Self::InvalidKeyType(e) => e.schema_name(),
            Self::InvalidSchemaName(e) => e.schema_name(),
            Self::InvalidServiceVersion(e) => e.schema_name(),
            Self::InvalidStructFieldId(e) => e.schema_name(),
            Self::InvalidSyntax(e) => e.schema_name(),
            Self::IoError(e) => e.schema_name(),
            Self::MissingImport(e) => e.schema_name(),
            Self::RecursiveEnum(e) => e.schema_name(),
            Self::RecursiveNewtype(e) => e.schema_name(),
            Self::RecursiveStruct(e) => e.schema_name(),
            Self::TypeNotFound(e) => e.schema_name(),
        }
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        match self {
            Self::ConstIntNotFound(e) => e.render(renderer, parser),
            Self::DuplicateDefinition(e) => e.render(renderer, parser),
            Self::DuplicateEnumVariant(e) => e.render(renderer, parser),
            Self::DuplicateEnumVariantId(e) => e.render(renderer, parser),
            Self::DuplicateEventId(e) => e.render(renderer, parser),
            Self::DuplicateFunctionId(e) => e.render(renderer, parser),
            Self::DuplicateServiceItem(e) => e.render(renderer, parser),
            Self::DuplicateServiceUuid(e) => e.render(renderer, parser),
            Self::DuplicateStructField(e) => e.render(renderer, parser),
            Self::DuplicateStructFieldId(e) => e.render(renderer, parser),
            Self::EmptyEnum(e) => e.render(renderer, parser),
            Self::ExpectedConstIntFoundService(e) => e.render(renderer, parser),
            Self::ExpectedConstIntFoundString(e) => e.render(renderer, parser),
            Self::ExpectedConstIntFoundType(e) => e.render(renderer, parser),
            Self::ExpectedConstIntFoundUuid(e) => e.render(renderer, parser),
            Self::ExpectedTypeFoundConst(e) => e.render(renderer, parser),
            Self::ExpectedTypeFoundService(e) => e.render(renderer, parser),
            Self::ImportNotFound(e) => e.render(renderer, parser),
            Self::InvalidArrayLen(e) => e.render(renderer, parser),
            Self::InvalidConstValue(e) => e.render(renderer, parser),
            Self::InvalidEnumVariantId(e) => e.render(renderer, parser),
            Self::InvalidEscapeCode(e) => e.render(renderer, parser),
            Self::InvalidEventId(e) => e.render(renderer, parser),
            Self::InvalidFunctionId(e) => e.render(renderer, parser),
            Self::InvalidIdent(e) => e.render(renderer, parser),
            Self::InvalidKeyType(e) => e.render(renderer, parser),
            Self::InvalidSchemaName(e) => e.render(renderer, parser),
            Self::InvalidServiceVersion(e) => e.render(renderer, parser),
            Self::InvalidStructFieldId(e) => e.render(renderer, parser),
            Self::InvalidSyntax(e) => e.render(renderer, parser),
            Self::IoError(e) => e.render(renderer, parser),
            Self::MissingImport(e) => e.render(renderer, parser),
            Self::RecursiveEnum(e) => e.render(renderer, parser),
            Self::RecursiveNewtype(e) => e.render(renderer, parser),
            Self::RecursiveStruct(e) => e.render(renderer, parser),
            Self::TypeNotFound(e) => e.render(renderer, parser),
        }
    }
}
