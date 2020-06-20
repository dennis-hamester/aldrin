use super::Warning;
use crate::ast::ImportStmt;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Schema, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateImport {
    schema_name: String,
    duplicate: ImportStmt,
    original_span: Span,
}

impl DuplicateImport {
    pub(crate) fn validate(schema: &Schema, validate: &mut Validate) {
        let mut imports = HashMap::new();
        for import in schema.imports() {
            match imports.entry(import.schema_name().value()) {
                Entry::Vacant(e) => {
                    e.insert(import);
                }
                Entry::Occupied(e) => {
                    validate.add_warning(DuplicateImport {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: import.clone(),
                        original_span: e.get().span(),
                    });
                }
            }
        }
    }

    pub fn duplicate(&self) -> &ImportStmt {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
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
        todo!()
    }
}

impl From<DuplicateImport> for Warning {
    fn from(w: DuplicateImport) -> Self {
        Warning::DuplicateImport(w)
    }
}
