use super::Error;
use crate::ast::{LitPosInt, ServiceDef, ServiceItem};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateFunctionId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
    free_id: u32,
}

impl DuplicateFunctionId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut ids = HashMap::new();

        let mut free_id = 1 + service
            .items()
            .iter()
            .filter_map(|i| match i {
                ServiceItem::Function(f) => Some(f),
                _ => None,
            })
            .fold(0, |cur, f| match f.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

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

    pub fn free_id(&self) -> u32 {
        self.free_id
    }
}

impl Diagnostic for DuplicateFunctionId {
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

impl From<DuplicateFunctionId> for Error {
    fn from(e: DuplicateFunctionId) -> Self {
        Error::DuplicateFunctionId(e)
    }
}
