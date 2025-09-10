use super::Error;
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ImportNotFound {
    schema_name: String,
    import: ImportStmt,
    tried: Vec<PathBuf>,
}

impl ImportNotFound {
    pub(crate) fn validate(import_stmt: &ImportStmt, validate: &mut Validate) {
        if validate
            .get_schema(import_stmt.schema_name().value())
            .is_some()
        {
            return;
        }

        let tried = validate
            .schema_paths()
            .iter()
            .map(|p| {
                let mut tried = p.clone();
                tried.push(import_stmt.schema_name().value());
                tried.set_extension("aldrin");
                tried
            })
            .collect();

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            import: import_stmt.clone(),
            tried,
        });
    }

    pub fn import(&self) -> &ImportStmt {
        &self.import
    }

    pub fn tried(&self) -> &[PathBuf] {
        &self.tried
    }
}

impl Diagnostic for ImportNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "file not found for schema `{}`",
                self.import.schema_name().value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.import.span().from, self.import.span(), "");
        }

        if self.tried.is_empty() {
            fmt.help("no include directories were specified");
        } else {
            fmt.help("an include directory may be missing or incorrect");
        }

        for tried in &self.tried {
            fmt.note(format!("tried `{}`", tried.display()));
        }

        fmt.format()
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(format!(
            "file not found for schema `{}`",
            self.import.schema_name().value()
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.import.span(), "");
        }

        if self.tried.is_empty() {
            report = report.help("no include directories were specified");
        } else {
            report = report.help("an include directory may be missing or incorrect");
        }

        for tried in &self.tried {
            report = report.note(format!("tried `{}`", tried.display()));
        }

        report.render()
    }
}

impl From<ImportNotFound> for Error {
    fn from(e: ImportNotFound) -> Self {
        Self::ImportNotFound(e)
    }
}
