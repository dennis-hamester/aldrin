use super::Error;
use crate::ast::{Ident, LitPosInt, ServiceDef, ServiceItem};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateEventId {
    schema_name: String,
    duplicate: LitPosInt,
    first: Span,
    service_ident: Ident,
    free_id: u32,
}

impl DuplicateEventId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let events = service.items().iter().filter_map(|item| match item {
            ServiceItem::Event(ev) => Some(ev),
            _ => None,
        });

        let mut max_id = events
            .clone()
            .fold(0, |cur, ev| match ev.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        util::find_duplicates(
            events,
            |ev| ev.id().value(),
            |duplicate, first| {
                max_id += 1;
                let free_id = max_id;
                validate.add_error(DuplicateEventId {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    service_ident: service.name().clone(),
                    free_id,
                })
            },
        );
    }

    pub fn duplicate(&self) -> &LitPosInt {
        &self.duplicate
    }

    pub fn first(&self) -> Span {
        self.first
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
        let mut fmt = Formatter::new(
            self,
            format!(
                "duplicate event id `{}` in service `{}`",
                self.duplicate.value(),
                self.service_ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.duplicate.span().from,
                self.duplicate.span(),
                "duplicate defined here",
            )
            .info_block(schema, self.first.from, self.first, "first defined here");
        }

        fmt.help(format!("use a free id, e.g. {}", self.free_id));
        fmt.format()
    }
}

impl From<DuplicateEventId> for Error {
    fn from(e: DuplicateEventId) -> Self {
        Error::DuplicateEventId(e)
    }
}
