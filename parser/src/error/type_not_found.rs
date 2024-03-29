use super::Error;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed};

#[derive(Debug)]
pub struct TypeNotFound {
    schema_name: String,
    ident: Ident,
    candidate: Option<String>,
}

impl TypeNotFound {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        let schema = validate.get_current_schema();
        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                return;
            }
        }

        let candidate = util::did_you_mean_type(schema, ident.value(), true).map(ToOwned::to_owned);

        validate.add_error(TypeNotFound {
            schema_name: validate.schema_name().to_owned(),
            ident: ident.clone(),
            candidate,
        });
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
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
        let mut fmt = Formatter::new(self, format!("type `{}` not found", self.ident.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.ident.span().from,
                self.ident.span(),
                "type used here",
            );
        }

        if let Some(ref candidate) = self.candidate {
            fmt.help(format!("did you mean `{candidate}`?"));
        }

        fmt.format()
    }
}

impl From<TypeNotFound> for Error {
    fn from(e: TypeNotFound) -> Self {
        Error::TypeNotFound(e)
    }
}
