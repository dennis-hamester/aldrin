use super::{Error, ErrorKind};
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser, Span};

#[derive(Debug)]
pub(crate) struct DuplicateServiceItem {
    schema_name: String,
    duplicate: Ident,
    first: Span,
    service_ident: Ident,
}

impl DuplicateServiceItem {
    pub(crate) fn validate(service: &ServiceDef, validate: &mut Validate) {
        let mut fallback_dup = false;

        util::find_duplicates(
            service.items(),
            |item| item.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    service_ident: service.name().clone(),
                })
            },
        );

        if let Some(fallback) = service.function_fallback() {
            for item in service.items() {
                if fallback.name().value() == item.name().value() {
                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: fallback.name().clone(),
                        first: item.name().span(),
                        service_ident: service.name().clone(),
                    });

                    fallback_dup = true;
                    break;
                }
            }
        }

        if let Some(fallback) = service.event_fallback() {
            for item in service.items() {
                if fallback.name().value() == item.name().value() {
                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: fallback.name().clone(),
                        first: item.name().span(),
                        service_ident: service.name().clone(),
                    });

                    fallback_dup = true;
                    break;
                }
            }
        }

        if !fallback_dup {
            if let (Some(func), Some(ev)) = (service.function_fallback(), service.event_fallback())
            {
                if func.name().value() == ev.name().value() {
                    let (duplicate, first) = if func.span().start < ev.span().end {
                        (ev.name().clone(), func.name().span())
                    } else {
                        (func.name().clone(), ev.name().span())
                    };

                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate,
                        first,
                        service_ident: service.name().clone(),
                    });
                }
            }
        }
    }
}

impl Diagnostic for DuplicateServiceItem {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "duplicate item `{}` in service `{}`",
            self.duplicate.value(),
            self.service_ident.value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate defined here")
                .context(schema, self.first, "first defined here");
        }

        report.render()
    }
}

impl From<DuplicateServiceItem> for Error {
    fn from(e: DuplicateServiceItem) -> Self {
        Self {
            kind: ErrorKind::DuplicateServiceItem(e),
        }
    }
}
