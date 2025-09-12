use super::{Error, ErrorKind};
use crate::ast::{FunctionDef, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parser;

#[derive(Debug)]
pub(crate) struct InvalidFunctionId {
    schema_name: String,
    id: LitPosInt,
    name_ident: Ident,
}

impl InvalidFunctionId {
    pub(crate) fn validate(func: &FunctionDef, validate: &mut Validate) {
        if func.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: func.id().clone(),
            name_ident: func.name().clone(),
        });
    }
}

impl Diagnostic for InvalidFunctionId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "invalid id `{}` for function `{}`",
            self.id.value(),
            self.name_ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.id.span(), "id defined here");
        }

        report = report.help("ids must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidFunctionId> for Error {
    fn from(e: InvalidFunctionId) -> Self {
        Self {
            kind: ErrorKind::InvalidFunctionId(e),
        }
    }
}
