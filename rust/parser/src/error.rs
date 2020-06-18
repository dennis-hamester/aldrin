mod duplicate_definition;
mod duplicate_struct_field;
mod duplicate_struct_field_id;
mod extern_type_not_found;
mod import_not_found;
mod invalid_const_value;
mod invalid_schema_name;
mod invalid_struct_field_id;
mod invalid_syntax;
mod io_error;
mod missing_import;
mod type_not_found;

pub use duplicate_definition::DuplicateDefinition;
pub use duplicate_struct_field::DuplicateStructField;
pub use duplicate_struct_field_id::DuplicateStructFieldId;
pub use extern_type_not_found::ExternTypeNotFound;
pub use import_not_found::ImportNotFound;
pub use invalid_const_value::InvalidConstValue;
pub use invalid_schema_name::InvalidSchemaName;
pub use invalid_struct_field_id::InvalidStructFieldId;
pub use invalid_syntax::{Expected, InvalidSyntax};
pub use io_error::IoError;
pub use missing_import::MissingImport;
pub use type_not_found::TypeNotFound;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    DuplicateDefinition(DuplicateDefinition),
    DuplicateStructField(DuplicateStructField),
    DuplicateStructFieldId(DuplicateStructFieldId),
    ExternTypeNotFound(ExternTypeNotFound),
    ImportNotFound(ImportNotFound),
    InvalidConstValue(InvalidConstValue),
    InvalidSchemaName(InvalidSchemaName),
    InvalidStructFieldId(InvalidStructFieldId),
    InvalidSyntax(InvalidSyntax),
    IoError(IoError),
    MissingImport(MissingImport),
    TypeNotFound(TypeNotFound),
}

impl Error {
    pub fn schema_name(&self) -> &str {
        match self {
            Error::DuplicateDefinition(e) => e.schema_name(),
            Error::DuplicateStructField(e) => e.schema_name(),
            Error::DuplicateStructFieldId(e) => e.schema_name(),
            Error::ExternTypeNotFound(e) => e.schema_name(),
            Error::ImportNotFound(e) => e.schema_name(),
            Error::InvalidConstValue(e) => e.schema_name(),
            Error::InvalidSchemaName(e) => e.schema_name(),
            Error::InvalidStructFieldId(e) => e.schema_name(),
            Error::InvalidSyntax(e) => e.schema_name(),
            Error::IoError(e) => e.schema_name(),
            Error::MissingImport(e) => e.schema_name(),
            Error::TypeNotFound(e) => e.schema_name(),
        }
    }
}
