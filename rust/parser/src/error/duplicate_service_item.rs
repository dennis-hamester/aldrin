use super::Error;
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateServiceItem {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
    service_ident: Ident,
}

impl DuplicateServiceItem {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut idents = HashMap::new();

        for item in service.items() {
            let name = item.name();
            match idents.entry(name.value()) {
                Entry::Vacant(e) => {
                    e.insert(name);
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateServiceItem {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: name.clone(),
                        original_span: e.get().span(),
                        service_ident: service.name().clone(),
                    });
                }
            }
        }
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
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
        let mut fmt = Formatter::error(format!(
            "duplicate item `{}` in service `{}`",
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

        fmt.format()
    }
}

impl From<DuplicateServiceItem> for Error {
    fn from(e: DuplicateServiceItem) -> Self {
        Error::DuplicateServiceItem(e)
    }
}
