use super::{Error, ErrorKind};
use crate::ast::{ConstValue, NamedRef, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser};

#[derive(Debug)]
pub(crate) struct ExpectedConstIntFoundUuid {
    schema_name: String,
    named_ref: NamedRef,
    candidate: Option<String>,
}

impl ExpectedConstIntFoundUuid {
    pub(crate) fn validate(named_ref: &NamedRef, validate: &mut Validate) {
        let (schema, ident) = match named_ref.kind() {
            NamedRefKind::Intern(ident) => (validate.get_current_schema(), ident),

            NamedRefKind::Extern(schema, ident) => {
                let Some(schema) = validate.get_schema(schema.value()) else {
                    return;
                };

                (schema, ident)
            }
        };

        let mut found = false;
        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                let Some(const_def) = def.as_const() else {
                    return;
                };

                if matches!(const_def.value(), ConstValue::Uuid(_)) {
                    found = true;
                } else {
                    return;
                }
            }
        }

        if found {
            let candidate =
                util::did_you_mean_const_int(schema, ident.value()).map(ToOwned::to_owned);

            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                named_ref: named_ref.clone(),
                candidate,
            });
        }
    }
}

impl Diagnostic for ExpectedConstIntFoundUuid {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "expected integer constant; found uuid constant `{}`",
            self.named_ref.ident().value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(
                schema,
                self.named_ref.span(),
                "integer constant expected here",
            );
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

impl From<ExpectedConstIntFoundUuid> for Error {
    fn from(e: ExpectedConstIntFoundUuid) -> Self {
        Self {
            kind: ErrorKind::ExpectedConstIntFoundUuid(e),
        }
    }
}
