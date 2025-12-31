use super::{Error, ErrorKind};
use crate::Parser;
use crate::ast::{Ident, LitInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;

#[derive(Debug)]
pub(crate) struct InvalidStructFieldId {
    schema_name: String,
    id: LitInt,
    field_ident: Ident,
}

impl InvalidStructFieldId {
    pub(crate) fn validate(field: &StructField, validate: &mut Validate) {
        if field.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: field.id().clone(),
            field_ident: field.name().clone(),
        });
    }
}

impl Diagnostic for InvalidStructFieldId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "invalid id `{}` for struct field `{}`",
            self.id.value(),
            self.field_ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.id.span(), "id defined here");
        }

        report = report.help("ids must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidStructFieldId> for Error {
    fn from(e: InvalidStructFieldId) -> Self {
        Self {
            kind: ErrorKind::InvalidStructFieldId(e),
        }
    }
}
