use super::Error;
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateServiceItem {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
}

impl DuplicateServiceItem {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut idents = HashMap::new();

        for item in service.items() {
            let name = item.name();
            match idents.entry(name.value()) {
                Entry::Vacant(e) => {
                    e.insert(name);
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateServiceItem {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: name.clone(),
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

impl Diagnostic for DuplicateServiceItem {
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

impl From<DuplicateServiceItem> for Error {
    fn from(e: DuplicateServiceItem) -> Self {
        Error::DuplicateServiceItem(e)
    }
}
