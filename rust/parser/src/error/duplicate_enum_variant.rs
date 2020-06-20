use super::Error;
use crate::ast::{EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateEnumVariant {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
}

impl DuplicateEnumVariant {
    pub(crate) fn validate(vars: &[EnumVariant], validate: &mut Validate) {
        let mut idents = HashMap::new();

        for var in vars {
            match idents.entry(var.name().value()) {
                Entry::Vacant(e) => {
                    e.insert(var.name());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateEnumVariant {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: var.name().clone(),
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

impl Diagnostic for DuplicateEnumVariant {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        todo!()
    }
}

impl From<DuplicateEnumVariant> for Error {
    fn from(e: DuplicateEnumVariant) -> Self {
        Error::DuplicateEnumVariant(e)
    }
}
