use super::{Error, ErrorKind};
use crate::ast::{EnumFallback, EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Span};

#[derive(Debug)]
pub(crate) struct EmptyEnum {
    schema_name: String,
    span: Span,
    ident: Option<Ident>,
}

impl EmptyEnum {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        fallback: Option<&EnumFallback>,
        span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        if !vars.is_empty() || fallback.is_some() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            span,
            ident: ident.cloned(),
        });
    }
}

impl Diagnostic for EmptyEnum {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = match self.ident {
            Some(ref ident) => renderer.error(format!("empty enum `{}`", ident.value())),
            None => renderer.error("empty inline enum"),
        };

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.span, "");
        }

        report = report
            .note("empty enums are not supported")
            .help("add at least one variant to the enum");

        report.render()
    }
}

impl From<EmptyEnum> for Error {
    fn from(e: EmptyEnum) -> Self {
        Self {
            kind: ErrorKind::EmptyEnum(e),
        }
    }
}
