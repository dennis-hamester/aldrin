use super::{Error, ErrorKind};
use crate::Parser;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;

#[derive(Debug)]
pub(crate) struct InvalidIdent {
    schema_name: String,
    ident: Ident,
}

impl InvalidIdent {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        if Ident::is_valid(ident.value()) {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ident: ident.clone(),
        });
    }
}

impl Diagnostic for InvalidIdent {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!("invalid identifier `{}`", self.ident.value()));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.note("identifiers must match [a-zA-Z_]+[0-9a-zA-Z_]*");
        report.render()
    }
}

impl From<InvalidIdent> for Error {
    fn from(e: InvalidIdent) -> Self {
        Self {
            kind: ErrorKind::InvalidIdent(e),
        }
    }
}
