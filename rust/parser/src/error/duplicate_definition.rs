use super::Error;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;
use crate::{Schema, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateDefinition {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
}

impl DuplicateDefinition {
    pub(crate) fn validate(schema: &Schema, validate: &mut Validate) {
        let mut idents = HashMap::new();

        for def in schema.definitions() {
            match idents.entry(def.name().value()) {
                Entry::Vacant(e) => {
                    e.insert(def.name());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateDefinition {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: def.name().clone(),
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

impl Diagnostic for DuplicateDefinition {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<DuplicateDefinition> for Error {
    fn from(e: DuplicateDefinition) -> Self {
        Error::DuplicateDefinition(e)
    }
}
