use super::{Error, ErrorKind};
use crate::ast::{Ident, StructFallback, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser, Span};

#[derive(Debug)]
pub(crate) struct DuplicateStructField {
    schema_name: String,
    duplicate: Ident,
    first: Span,
    struct_ident: Option<Ident>,
}

impl DuplicateStructField {
    pub(crate) fn validate(
        fields: &[StructField],
        fallback: Option<&StructFallback>,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        util::find_duplicates(
            fields,
            |field| field.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    struct_ident: ident.cloned(),
                })
            },
        );

        if let Some(fallback) = fallback {
            for field in fields {
                if fallback.name().value() == field.name().value() {
                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: fallback.name().clone(),
                        first: field.name().span(),
                        struct_ident: ident.cloned(),
                    });

                    break;
                }
            }
        }
    }
}

impl Diagnostic for DuplicateStructField {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let title = match self.struct_ident {
            Some(ref ident) => format!(
                "duplicate field `{}` in struct `{}`",
                self.duplicate.value(),
                ident.value()
            ),

            None => format!(
                "duplicate field `{}` in inline struct",
                self.duplicate.value()
            ),
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

impl From<DuplicateStructField> for Error {
    fn from(e: DuplicateStructField) -> Self {
        Self {
            kind: ErrorKind::DuplicateStructField(e),
        }
    }
}
