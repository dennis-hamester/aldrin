use super::{Error, ErrorKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::Parser;

#[derive(Debug)]
pub(crate) struct IoError {
    schema_name: String,
    err: String,
}

impl IoError {
    pub(crate) fn new<S>(schema_name: S, err: String) -> Self
    where
        S: Into<String>,
    {
        Self {
            schema_name: schema_name.into(),
            err,
        }
    }
}

impl Diagnostic for IoError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(&self.err);

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.note(format!("tried to read `{}`", schema.path()));
        }

        report.render()
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self {
            kind: ErrorKind::IoError(e),
        }
    }
}
