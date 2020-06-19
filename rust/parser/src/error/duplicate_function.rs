use super::Error;
use crate::ast::{Ident, ServiceDef, ServiceItem};
use crate::validate::Validate;
use crate::Span;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateFunction {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
}

impl DuplicateFunction {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut idents = HashMap::new();

        for item in service.items() {
            let func = match item {
                ServiceItem::Function(func) => func,
                _ => continue,
            };

            match idents.entry(func.name().value()) {
                Entry::Vacant(e) => {
                    e.insert(func.name());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateFunction {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: func.name().clone(),
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

impl From<DuplicateFunction> for Error {
    fn from(e: DuplicateFunction) -> Self {
        Error::DuplicateFunction(e)
    }
}
