use super::Error;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;
use crate::Span;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateStructField {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
}

impl DuplicateStructField {
    pub(crate) fn validate(fields: &[StructField], validate: &mut Validate) {
        let mut idents = HashMap::new();

        for field in fields {
            match idents.entry(field.name().value()) {
                Entry::Vacant(e) => {
                    e.insert(field.name());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateStructField {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: field.name().clone(),
                        original_span: e.get().span(),
                    });
                }
            }
        }
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
    }
}

impl Diagnostic for DuplicateStructField {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<DuplicateStructField> for Error {
    fn from(e: DuplicateStructField) -> Self {
        Error::DuplicateStructField(e)
    }
}
