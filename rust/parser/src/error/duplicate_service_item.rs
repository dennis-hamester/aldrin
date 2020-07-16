use super::Error;
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateServiceItem {
    schema_name: String,
    duplicate: Ident,
    first: Span,
    service_ident: Ident,
}

impl DuplicateServiceItem {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        util::find_duplicates(
            service.items(),
            |item| item.name().value(),
            |duplicate, first| {
                validate.add_error(DuplicateServiceItem {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    service_ident: service.name().clone(),
                })
            },
        );
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn first(&self) -> Span {
        self.first
    }

    pub fn service_ident(&self) -> &Ident {
        &self.service_ident
    }
}

impl Diagnostic for DuplicateServiceItem {
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
                "duplicate item `{}` in service `{}`",
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

        fmt.format()
    }
}

impl From<DuplicateServiceItem> for Error {
    fn from(e: DuplicateServiceItem) -> Self {
        Error::DuplicateServiceItem(e)
    }
}
