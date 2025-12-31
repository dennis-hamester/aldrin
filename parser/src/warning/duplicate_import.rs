use super::{Warning, WarningKind};
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Schema, Span, util};

#[derive(Debug)]
pub(crate) struct DuplicateImport {
    schema_name: String,
    duplicate: ImportStmt,
    first: Span,
}

impl DuplicateImport {
    pub(crate) fn validate(schema: &Schema, validate: &mut Validate) {
        util::find_duplicates(
            schema.imports(),
            |import| import.schema_name().value(),
            |duplicate, first| {
                validate.add_warning(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.clone(),
                    first: first.span(),
                });
            },
        );
    }
}

impl Diagnostic for DuplicateImport {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "duplicate import of schema `{}`",
            self.duplicate.schema_name().value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate import")
                .context(schema, self.first, "first imported here");
        }

        report = report.help("remove the duplicate import statement");
        report.render()
    }
}

impl From<DuplicateImport> for Warning {
    fn from(w: DuplicateImport) -> Self {
        Self {
            kind: WarningKind::DuplicateImport(w),
        }
    }
}
