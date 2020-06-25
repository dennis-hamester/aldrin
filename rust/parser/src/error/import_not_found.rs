use super::Error;
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use std::env;
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

        validate.add_error(ImportNotFound {
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
        let mut fmt = Formatter::error(format!(
            "file not found for schema `{}`",
            self.import.schema_name().value()
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.import.span().from, self.import.span(), "");
        }

        for tried in &self.tried {
            if tried.is_absolute() {
                fmt.note(format!("tried `{}`", tried.display()));
            } else if let Ok(mut absolute) = env::current_dir() {
                absolute.push(tried);
                fmt.note(format!("tried `{}`", absolute.display()));
            } else {
                fmt.note(format!("tried `{}`", tried.display()));
            }
        }

        if self.tried.is_empty() {
            fmt.note("an include directory may be missing");
        } else {
            fmt.note("an include directory may be missing or incorrect");
        }
        fmt.format()
    }
}

impl From<ImportNotFound> for Error {
    fn from(e: ImportNotFound) -> Self {
        Error::ImportNotFound(e)
    }
}
