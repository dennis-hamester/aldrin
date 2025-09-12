use super::{Error, ErrorKind};
use crate::ast::{NamedRef, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser};

#[derive(Debug)]
pub(crate) struct ExpectedTypeFoundConst {
    schema_name: String,
    named_ref: NamedRef,
    candidate: Option<String>,
}

impl ExpectedTypeFoundConst {
    pub(crate) fn validate(named_ref: &NamedRef, is_key_type: bool, validate: &mut Validate) {
        let (schema, ident, intern) = match named_ref.kind() {
            NamedRefKind::Intern(ident) => (validate.get_current_schema(), ident, true),

            NamedRefKind::Extern(schema, ident) => {
                let Some(schema) = validate.get_schema(schema.value()) else {
                    return;
                };

                (schema, ident, false)
            }
        };

        let mut found = false;
        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                if def.as_const().is_some() {
                    found = true;
                } else {
                    return;
                }
            }
        }

        if found {
            let candidate = if is_key_type {
                util::did_you_mean_key_type(schema, ident.value(), intern, validate)
                    .map(ToOwned::to_owned)
            } else {
                util::did_you_mean_type(schema, ident.value(), intern).map(ToOwned::to_owned)
            };

            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                named_ref: named_ref.clone(),
                candidate,
            });
        }
    }
}

impl Diagnostic for ExpectedTypeFoundConst {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "expected type; found constant `{}`",
            self.named_ref.ident().value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.named_ref.span(), "type expected here");
        }

        if let Some(ref candidate) = self.candidate {
            let msg = match self.named_ref.schema() {
                Some(schema) => format!("did you mean `{}::{candidate}`?", schema.value()),
                None => format!("did you mean `{candidate}`?"),
            };

            report = report.help(msg);
        }

        report.render()
    }
}

impl From<ExpectedTypeFoundConst> for Error {
    fn from(e: ExpectedTypeFoundConst) -> Self {
        Self {
            kind: ErrorKind::ExpectedTypeFoundConst(e),
        }
    }
}
