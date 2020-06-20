use super::Error;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateStructField {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
    struct_span: Span,
    struct_ident: Option<Ident>,
}

impl DuplicateStructField {
    pub(crate) fn validate(
        fields: &[StructField],
        struct_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
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
                        struct_span,
                        struct_ident: ident.cloned(),
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

    pub fn struct_span(&self) -> Span {
        self.struct_span
    }

    pub fn struct_ident(&self) -> Option<&Ident> {
        self.struct_ident.as_ref()
    }
}

impl Diagnostic for DuplicateStructField {
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

impl From<DuplicateStructField> for Error {
    fn from(e: DuplicateStructField) -> Self {
        Error::DuplicateStructField(e)
    }
}
