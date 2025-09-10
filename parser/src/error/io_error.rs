use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::Parsed;

#[derive(Debug)]
pub struct IoError {
    schema_name: String,
    err: std::io::Error,
}

impl IoError {
    pub(crate) fn new<S>(schema_name: S, err: std::io::Error) -> Self
    where
        S: Into<String>,
    {
        Self {
            schema_name: schema_name.into(),
            err,
        }
    }

    pub fn io_error(&self) -> &std::io::Error {
        &self.err
    }
}

impl Diagnostic for IoError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(self.err.to_string());

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.note(format!("tried to read `{}`", schema.path().display()));
        }

        report.render()
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::IoError(e)
    }
}
