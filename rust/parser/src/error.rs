mod invalid_const_value;
mod invalid_schema_name;
mod io_error;
mod parser_error;

pub use invalid_const_value::InvalidConstValue;
pub use invalid_schema_name::InvalidSchemaName;
pub use io_error::IoError;
pub use parser_error::{Expected, ParserError};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Io(IoError),
    Parser(ParserError),
    InvalidSchemaName(InvalidSchemaName),
    InvalidConstValue(InvalidConstValue),
}

impl Error {
    pub fn schema_name(&self) -> &str {
        match self {
            Error::Io(e) => e.schema_name(),
            Error::Parser(e) => e.schema_name(),
            Error::InvalidSchemaName(e) => e.schema_name(),
            Error::InvalidConstValue(e) => e.schema_name(),
        }
    }
}
