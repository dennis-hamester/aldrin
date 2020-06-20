use super::Error;
use crate::ast::{Ident, LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateStructFieldId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
    struct_span: Span,
    struct_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateStructFieldId {
    pub(crate) fn validate(
        fields: &[StructField],
        struct_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        let mut ids = HashMap::new();

        let mut free_id = 1 + fields
            .iter()
            .fold(0, |cur, f| match f.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        for field in fields {
            match ids.entry(field.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(field.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateStructFieldId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: field.id().clone(),
                        original_span: e.get().span(),
                        struct_span,
                        struct_ident: ident.cloned(),
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

    pub fn struct_span(&self) -> Span {
        self.struct_span
    }

    pub fn struct_ident(&self) -> Option<&Ident> {
        self.struct_ident.as_ref()
    }

    pub fn free_id(&self) -> u32 {
        self.free_id
    }
}

impl Diagnostic for DuplicateStructFieldId {
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

impl From<DuplicateStructFieldId> for Error {
    fn from(e: DuplicateStructFieldId) -> Self {
        Error::DuplicateStructFieldId(e)
    }
}
