use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind};

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
        IoError {
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
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}
