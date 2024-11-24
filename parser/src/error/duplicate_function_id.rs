use super::Error;
use crate::ast::{Ident, LitPosInt, ServiceDef, ServiceItem};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateFunctionId {
    schema_name: String,
    duplicate: LitPosInt,
    first: Span,
    service_ident: Ident,
    free_id: u32,
}

impl DuplicateFunctionId {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let funcs = service.items().iter().filter_map(|item| match item {
            ServiceItem::Function(ev) => Some(ev),
            _ => None,
        });

        let mut max_id = funcs
            .clone()
            .fold(0, |cur, ev| match ev.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        util::find_duplicates(
            funcs,
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

impl Diagnostic for DuplicateFunctionId {
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
                "duplicate function id `{}` in service `{}`",
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

impl From<DuplicateFunctionId> for Error {
    fn from(e: DuplicateFunctionId) -> Self {
        Self::DuplicateFunctionId(e)
    }
}
