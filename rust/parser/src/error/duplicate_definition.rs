use super::Error;
use crate::ast::Ident;
use crate::validate::Validate;
use crate::{Definition, Schema, Span};
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
            let ident = match def {
                Definition::Const(d) => d.name(),
                Definition::Struct(d) => d.name(),
            };

            match idents.entry(ident.value()) {
                Entry::Vacant(e) => {
                    e.insert(ident.clone());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateDefinition {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: ident.clone(),
                        original_span: e.get().span(),
                    });
                }
            }
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
    }
}

impl From<DuplicateDefinition> for Error {
    fn from(e: DuplicateDefinition) -> Self {
        Error::DuplicateDefinition(e)
    }
}
