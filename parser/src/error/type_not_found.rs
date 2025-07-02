use super::Error;
use crate::ast::{NamedRef, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed};

#[derive(Debug)]
pub struct TypeNotFound {
    schema_name: String,
    named_ref: NamedRef,
    candidate: Option<String>,
}

impl TypeNotFound {
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

        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                return;
            }
        }

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

    pub fn named_ref(&self) -> &NamedRef {
        &self.named_ref
    }

    pub fn candidate(&self) -> Option<&str> {
        self.candidate.as_deref()
    }
}

impl Diagnostic for TypeNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(self, format!("type `{}` not found", self.named_ref.kind()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.named_ref.span().from,
                self.named_ref.span(),
                "type used here",
            );
        }

        if let Some(ref candidate) = self.candidate {
            match self.named_ref.schema() {
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
}

impl From<TypeNotFound> for Error {
    fn from(e: TypeNotFound) -> Self {
        Self::TypeNotFound(e)
    }
}
