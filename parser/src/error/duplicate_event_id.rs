use super::{Error, ErrorKind};
use crate::ast::{Ident, LitInt, ServiceDef, ServiceItem};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Span, util};

#[derive(Debug)]
pub(crate) struct DuplicateEventId {
    schema_name: String,
    duplicate: LitInt,
    first: Span,
    service_ident: Ident,
    free_id: u32,
}

impl DuplicateEventId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let events = service.items().iter().filter_map(ServiceItem::as_event);

        let mut max_id = events
            .clone()
            .filter_map(|ev| ev.id().value().parse().ok())
            .max()
            .unwrap_or(0);

        util::find_duplicates(
            events.filter(|ev| ev.id().value().parse::<u32>().is_ok()),
            |ev| ev.id().value(),
            |duplicate, first| {
                max_id += 1;
                let free_id = max_id;
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    service_ident: service.name().clone(),
                    free_id,
                });
            },
        );
    }
}

impl Diagnostic for DuplicateEventId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "duplicate event id `{}` in service `{}`",
            self.duplicate.value(),
            self.service_ident.value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate defined here")
                .context(schema, self.first, "first defined here");
        }

        report = report.help(format!("use a free id, e.g. {}", self.free_id));
        report.render()
    }
}

impl From<DuplicateEventId> for Error {
    fn from(e: DuplicateEventId) -> Self {
        Self {
            kind: ErrorKind::DuplicateEventId(e),
        }
    }
}
