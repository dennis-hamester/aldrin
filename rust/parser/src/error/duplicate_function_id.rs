use super::Error;
use crate::ast::{LitPosInt, ServiceDef, ServiceItem};
use crate::validate::Validate;
use crate::Span;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateFunctionId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
}

impl DuplicateFunctionId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut ids = HashMap::new();

        for item in service.items() {
            let func = match item {
                ServiceItem::Function(func) => func,
                _ => continue,
            };

            match ids.entry(func.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(func.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateFunctionId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: func.id().clone(),
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

impl From<DuplicateFunctionId> for Error {
    fn from(e: DuplicateFunctionId) -> Self {
        Error::DuplicateFunctionId(e)
    }
}
