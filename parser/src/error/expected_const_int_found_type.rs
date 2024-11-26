use super::Error;
use crate::ast::{Definition, NamedRef, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed};

#[derive(Debug)]
pub struct ExpectedConstIntFoundType {
    schema_name: String,
    named_ref: NamedRef,
    candidate: Option<String>,
}

impl ExpectedConstIntFoundType {
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
                match def {
                    Definition::Struct(_) | Definition::Enum(_) => found = true,
                    Definition::Service(_) | Definition::Const(_) => return,
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

    pub fn named_ref(&self) -> &NamedRef {
        &self.named_ref
    }
}

impl Diagnostic for ExpectedConstIntFoundType {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "expected integer constant; found type `{}`",
                self.named_ref.ident().value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.named_ref.span().from,
                self.named_ref.span(),
                "integer constant expected here",
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

impl From<ExpectedConstIntFoundType> for Error {
    fn from(e: ExpectedConstIntFoundType) -> Self {
        Self::ExpectedConstIntFoundType(e)
    }
}