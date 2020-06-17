mod duplicate_definition;
mod import_not_found;
mod invalid_const_value;
mod invalid_schema_name;
mod io_error;
mod parser_error;

pub use duplicate_definition::DuplicateDefinition;
pub use import_not_found::ImportNotFound;
pub use invalid_const_value::InvalidConstValue;
pub use invalid_schema_name::InvalidSchemaName;
pub use io_error::IoError;
pub use parser_error::{Expected, ParserError};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    DuplicateDefinition(DuplicateDefinition),
    ImportNotFound(ImportNotFound),
    InvalidConstValue(InvalidConstValue),
    InvalidSchemaName(InvalidSchemaName),
    Io(IoError),
    Parser(ParserError),
}

impl Error {
    pub fn schema_name(&self) -> &str {
        match self {
            Error::DuplicateDefinition(e) => e.schema_name(),
            Error::ImportNotFound(e) => e.schema_name(),
            Error::InvalidConstValue(e) => e.schema_name(),
            Error::InvalidSchemaName(e) => e.schema_name(),
            Error::Io(e) => e.schema_name(),
            Error::Parser(e) => e.schema_name(),
        }
    }
}
