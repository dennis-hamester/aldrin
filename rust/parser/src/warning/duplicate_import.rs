use super::Warning;
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Schema, Span};

#[derive(Debug)]
pub struct DuplicateImport {
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
                validate.add_warning(DuplicateImport {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.clone(),
                    first: first.span(),
                });
            },
        );
    }

    pub fn duplicate(&self) -> &ImportStmt {
        &self.duplicate
    }

    pub fn first(&self) -> Span {
        self.first
    }
}

impl Diagnostic for DuplicateImport {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::warning(format!(
            "duplicate import of schema `{}`",
            self.duplicate.schema_name().value()
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.duplicate.span().from,
                self.duplicate.span(),
                "duplicate import",
            );
            fmt.info_block(schema, self.first.from, self.first, "first imported here");
        }

        fmt.help("remove the duplicate import statement");
        fmt.format()
    }
}

impl From<DuplicateImport> for Warning {
    fn from(w: DuplicateImport) -> Self {
        Warning::DuplicateImport(w)
    }
}
