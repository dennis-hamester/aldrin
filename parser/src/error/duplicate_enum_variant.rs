use super::{Error, ErrorKind};
use crate::ast::{EnumFallback, EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Span, util};

#[derive(Debug)]
pub(crate) struct DuplicateEnumVariant {
    schema_name: String,
    duplicate: Ident,
    first: Span,
    enum_ident: Option<Ident>,
}

impl DuplicateEnumVariant {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        fallback: Option<&EnumFallback>,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        util::find_duplicates(
            vars,
            |var| var.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    enum_ident: ident.cloned(),
                })
            },
        );

        if let Some(fallback) = fallback {
            for var in vars {
                if fallback.name().value() == var.name().value() {
                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: fallback.name().clone(),
                        first: var.span(),
                        enum_ident: ident.cloned(),
                    });

                    break;
                }
            }
        }
    }
}

impl Diagnostic for DuplicateEnumVariant {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let title = if let Some(ref ident) = self.enum_ident {
            format!(
                "duplicate variant `{}` in enum `{}`",
                self.duplicate.value(),
                ident.value()
            )
        } else {
            format!(
                "duplicate variant `{}` in inline enum",
                self.duplicate.value(),
            )
        };

        let mut report = renderer.error(title);

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate defined here")
                .context(schema, self.first, "first defined here");
        }

        report.render()
    }
}

impl From<DuplicateEnumVariant> for Error {
    fn from(e: DuplicateEnumVariant) -> Self {
        Self {
            kind: ErrorKind::DuplicateEnumVariant(e),
        }
    }
}
