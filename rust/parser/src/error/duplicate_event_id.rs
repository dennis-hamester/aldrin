use super::Error;
use crate::ast::{Ident, LitPosInt, ServiceDef, ServiceItem};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateEventId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
    service_ident: Ident,
    free_id: u32,
}

impl DuplicateEventId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut ids = HashMap::new();

        let mut free_id = 1 + service
            .items()
            .iter()
            .filter_map(|i| match i {
                ServiceItem::Event(e) => Some(e),
                _ => None,
            })
            .fold(0, |cur, e| match e.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        for item in service.items() {
            let ev = match item {
                ServiceItem::Event(ev) => ev,
                _ => continue,
            };

            match ids.entry(ev.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(ev.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateEventId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: ev.id().clone(),
                        original_span: e.get().span(),
                        service_ident: service.name().clone(),
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

    pub fn service_ident(&self) -> &Ident {
        &self.service_ident
    }

    pub fn free_id(&self) -> u32 {
        self.free_id
    }
}

impl Diagnostic for DuplicateEventId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::error(format!(
            "duplicate event id `{}` in service `{}`",
            self.duplicate.value(),
            self.service_ident.value()
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.duplicate.span().from,
                self.duplicate.span(),
                "duplicate defined here",
            )
            .info_block(
                schema,
                self.original_span.from,
                self.original_span,
                "first defined here",
            )
            .info_block(
                schema,
                self.service_ident.span().from,
                self.service_ident.span(),
                format!("service `{}` defined here", self.service_ident.value()),
            );
        }

        fmt.help(format!("use a free id like {}", self.free_id));
        fmt.format()
    }
}

impl From<DuplicateEventId> for Error {
    fn from(e: DuplicateEventId) -> Self {
        Error::DuplicateEventId(e)
    }
}
