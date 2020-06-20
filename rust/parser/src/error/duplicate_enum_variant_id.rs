use super::Error;
use crate::ast::{EnumVariant, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateEnumVariantId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
    enum_span: Span,
    enum_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateEnumVariantId {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        enum_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        let mut ids = HashMap::new();

        let mut free_id = 1 + vars.iter().fold(0, |cur, v| match v.id().value().parse() {
            Ok(id) if id > cur => id,
            _ => cur,
        });

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
                        enum_span,
                        enum_ident: ident.cloned(),
                        free_id,
                    });

                    free_id += 1;
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

    pub fn enum_span(&self) -> Span {
        self.enum_span
    }

    pub fn enum_ident(&self) -> Option<&Ident> {
        self.enum_ident.as_ref()
    }

    pub fn free_id(&self) -> u32 {
        self.free_id
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
