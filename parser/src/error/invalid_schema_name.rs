use super::{Error, ErrorKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::Parser;

#[derive(Debug)]
pub(crate) struct InvalidSchemaName {
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

    fn render(&self, renderer: &Renderer, _parser: &Parser) -> String {
        let mut report = renderer.error(format!("invalid schema name `{}`", self.schema_name));

        if self.schema_name.contains('-') {
            report = report.help("hyphens `-` are not allowed in schema names");
        }

        report = report.note("schema names must match [a-zA-Z_]+[0-9a-zA-Z_]*");
        report.render()
    }
}

impl From<InvalidSchemaName> for Error {
    fn from(e: InvalidSchemaName) -> Self {
        Self {
            kind: ErrorKind::InvalidSchemaName(e),
        }
    }
}
