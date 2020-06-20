use super::Error;
use crate::ast::{EnumVariant, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateEnumVariantId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
}

impl DuplicateEnumVariantId {
    pub(crate) fn validate(vars: &[EnumVariant], validate: &mut Validate) {
        let mut ids = HashMap::new();

        for var in vars {
            match ids.entry(var.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(var.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateEnumVariantId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: var.id().clone(),
                        original_span: e.get().span(),
                    });
                }
            }
        }
    }

    pub fn duplicate(&self) -> &LitPosInt {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
    }
}

impl Diagnostic for DuplicateEnumVariantId {
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

impl From<DuplicateEnumVariantId> for Error {
    fn from(e: DuplicateEnumVariantId) -> Self {
        Error::DuplicateEnumVariantId(e)
    }
}
