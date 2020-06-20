use super::Warning;
use crate::diag::{Diagnostic, DiagnosticKind};

#[derive(Debug)]
pub struct NonSnakeCaseSchemaName {
    schema_name: String,
    snake_case: String,
}

impl NonSnakeCaseSchemaName {
    pub(crate) fn new<S1, S2>(schema_name: S1, snake_case: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        NonSnakeCaseSchemaName {
            schema_name: schema_name.into(),
            snake_case: snake_case.into(),
        }
    }

    pub fn snake_case(&self) -> &str {
        &self.snake_case
    }
}

impl Diagnostic for NonSnakeCaseSchemaName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<NonSnakeCaseSchemaName> for Warning {
    fn from(w: NonSnakeCaseSchemaName) -> Self {
        Warning::NonSnakeCaseSchemaName(w)
    }
}
