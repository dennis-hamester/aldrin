use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidSchemaName {
    schema_name: String,
}

impl InvalidSchemaName {
    pub(crate) fn new<S>(schema_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
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

    fn format<'a>(&'a self, _parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(self, format!("invalid schema name `{}`", self.schema_name));

        fmt.note("schema names are parsed from the file name");

        if self.schema_name.contains('-') {
            fmt.note_cont("hyphens `-` are not allowed in schema names");
        }

        fmt.format()
    }
}

impl From<InvalidSchemaName> for Error {
    fn from(e: InvalidSchemaName) -> Self {
        Self::InvalidSchemaName(e)
    }
}
