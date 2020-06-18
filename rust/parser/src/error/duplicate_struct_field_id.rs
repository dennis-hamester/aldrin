use super::Error;
use crate::ast::{LitPosInt, StructField};
use crate::validate::Validate;
use crate::Span;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateStructFieldId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
}

impl DuplicateStructFieldId {
    pub(crate) fn validate(fields: &[StructField], validate: &mut Validate) {
        let mut idents = HashMap::new();

        for field in fields {
            match idents.entry(field.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(field.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateStructFieldId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: field.id().clone(),
                        original_span: e.get().span(),
                    });
                }
            }
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn duplicate(&self) -> &LitPosInt {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
    }
}

impl From<DuplicateStructFieldId> for Error {
    fn from(e: DuplicateStructFieldId) -> Self {
        Error::DuplicateStructFieldId(e)
    }
}