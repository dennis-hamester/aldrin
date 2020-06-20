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
    enum_span: Span,
    enum_ident: Option<Ident>,
}

impl DuplicateEnumVariant {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        enum_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
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
                        enum_span,
                        enum_ident: ident.cloned(),
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

    pub fn enum_span(&self) -> Span {
        self.enum_span
    }

    pub fn enum_ident(&self) -> Option<&Ident> {
        self.enum_ident.as_ref()
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
