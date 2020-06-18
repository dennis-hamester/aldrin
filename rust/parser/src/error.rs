mod duplicate_definition;
mod duplicate_struct_field;
mod duplicate_struct_field_id;
mod import_not_found;
mod invalid_const_value;
mod invalid_schema_name;
mod invalid_struct_field_id;
mod io_error;
mod parser_error;

pub use duplicate_definition::DuplicateDefinition;
pub use duplicate_struct_field::DuplicateStructField;
pub use duplicate_struct_field_id::DuplicateStructFieldId;
pub use import_not_found::ImportNotFound;
pub use invalid_const_value::InvalidConstValue;
pub use invalid_schema_name::InvalidSchemaName;
pub use invalid_struct_field_id::InvalidStructFieldId;
pub use io_error::IoError;
pub use parser_error::{Expected, ParserError};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    DuplicateDefinition(DuplicateDefinition),
    DuplicateStructField(DuplicateStructField),
    DuplicateStructFieldId(DuplicateStructFieldId),
    ImportNotFound(ImportNotFound),
    InvalidConstValue(InvalidConstValue),
    InvalidSchemaName(InvalidSchemaName),
    InvalidStructFieldId(InvalidStructFieldId),
    Io(IoError),
    Parser(ParserError),
}

impl Error {
    pub fn schema_name(&self) -> &str {
        match self {
            Error::DuplicateDefinition(e) => e.schema_name(),
            Error::DuplicateStructField(e) => e.schema_name(),
            Error::DuplicateStructFieldId(e) => e.schema_name(),
            Error::ImportNotFound(e) => e.schema_name(),
            Error::InvalidConstValue(e) => e.schema_name(),
            Error::InvalidSchemaName(e) => e.schema_name(),
            Error::InvalidStructFieldId(e) => e.schema_name(),
            Error::Io(e) => e.schema_name(),
            Error::Parser(e) => e.schema_name(),
        }
    }
}
