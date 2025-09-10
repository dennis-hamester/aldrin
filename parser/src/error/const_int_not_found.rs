use super::Error;
use crate::ast::{NamedRef, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter, Renderer};
use crate::validate::Validate;
use crate::{util, Parsed};

#[derive(Debug)]
pub struct ConstIntNotFound {
    schema_name: String,
    named_ref: NamedRef,
    candidate: Option<String>,
}

impl ConstIntNotFound {
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

        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                return;
            }
        }

        let candidate = util::did_you_mean_const_int(schema, ident.value()).map(ToOwned::to_owned);

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            named_ref: named_ref.clone(),
            candidate,
        });
    }

    pub fn named_ref(&self) -> &NamedRef {
        &self.named_ref
    }

    pub fn candidate(&self) -> Option<&str> {
        self.candidate.as_deref()
    }
}

impl Diagnostic for ConstIntNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let (mut fmt, schema) = match self.named_ref.kind() {
            NamedRefKind::Intern(ident) => (
                Formatter::new(
                    self,
                    format!("integer constant `{}` not found", ident.value()),
                ),
                None,
            ),

            NamedRefKind::Extern(schema, ident) => (
                Formatter::new(
                    self,
                    format!(
                        "integer constant `{}::{}` not found",
                        schema.value(),
                        ident.value()
                    ),
                ),
                Some(schema),
            ),
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.named_ref.span().from,
                self.named_ref.span(),
                "integer constant used here",
            );
        }

        if let Some(ref candidate) = self.candidate {
            match schema {
                Some(schema) => {
                    fmt.help(format!("did you mean `{}::{candidate}`?", schema.value()));
                }

                None => {
                    fmt.help(format!("did you mean `{candidate}`?"));
                }
            }
        }

        fmt.format()
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let (title, schema) = match self.named_ref.kind() {
            NamedRefKind::Intern(ident) => (
                format!("integer constant `{}` not found", ident.value()),
                None,
            ),

            NamedRefKind::Extern(schema, ident) => (
                format!(
                    "integer constant `{}::{}` not found",
                    schema.value(),
                    ident.value()
                ),
                Some(schema),
            ),
        };

        let mut report = renderer.error(title);

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.named_ref.span(), "integer constant used here");
        }

        if let Some(ref candidate) = self.candidate {
            let msg = match schema {
                Some(schema) => format!("did you mean `{}::{candidate}`?", schema.value()),
                None => format!("did you mean `{candidate}`?"),
            };

            report = report.help(msg);
        }

        report.render()
    }
}

impl From<ConstIntNotFound> for Error {
    fn from(e: ConstIntNotFound) -> Self {
        Self::ConstIntNotFound(e)
    }
}
