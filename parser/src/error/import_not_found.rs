use super::{Error, ErrorKind};
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parser;

#[derive(Debug)]
pub(crate) struct ImportNotFound {
    schema_name: String,
    import: ImportStmt,
}

impl ImportNotFound {
    pub(crate) fn validate(import_stmt: &ImportStmt, validate: &mut Validate) {
        if validate
            .get_schema(import_stmt.schema_name().value())
            .is_some()
        {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            import: import_stmt.clone(),
        });
    }
}

impl Diagnostic for ImportNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "schema `{}` not found",
            self.import.schema_name().value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.import.span(), "");
        }

        report = report.help("an include directory may be missing or incorrect");
        report.render()
    }
}

impl From<ImportNotFound> for Error {
    fn from(e: ImportNotFound) -> Self {
        Self {
            kind: ErrorKind::ImportNotFound(e),
        }
    }
}
