use super::{Error, ErrorKind};
use crate::Parser;
use crate::ast::{EnumVariant, Ident, LitInt};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;

#[derive(Debug)]
pub(crate) struct InvalidEnumVariantId {
    schema_name: String,
    id: LitInt,
    var_ident: Ident,
}

impl InvalidEnumVariantId {
    pub(crate) fn validate(var: &EnumVariant, validate: &mut Validate) {
        if var.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: var.id().clone(),
            var_ident: var.name().clone(),
        });
    }
}

impl Diagnostic for InvalidEnumVariantId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "invalid id `{}` for enum variant `{}`",
            self.id.value(),
            self.var_ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.id.span(), "id defined here");
        }

        report = report.help("ids must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidEnumVariantId> for Error {
    fn from(e: InvalidEnumVariantId) -> Self {
        Self {
            kind: ErrorKind::InvalidEnumVariantId(e),
        }
    }
}
