use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind};

#[derive(Debug)]
pub struct InvalidSchemaName {
    schema_name: String,
}

impl InvalidSchemaName {
    pub(crate) fn new<S>(schema_name: S) -> Self
    where
        S: Into<String>,
    {
        InvalidSchemaName {
            schema_name: schema_name.into(),
        }
    }
}

impl Diagnostic for InvalidSchemaName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<InvalidSchemaName> for Error {
    fn from(e: InvalidSchemaName) -> Self {
        Error::InvalidSchemaName(e)
    }
}
