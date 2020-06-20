use super::Error;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct TypeNotFound {
    schema_name: String,
    ident: Ident,
}

impl TypeNotFound {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        let schema = validate.get_current_schema();
        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                return;
            }
        }

        validate.add_error(TypeNotFound {
            schema_name: validate.schema_name().to_owned(),
            ident: ident.clone(),
        });
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for TypeNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<TypeNotFound> for Error {
    fn from(e: TypeNotFound) -> Self {
        Error::TypeNotFound(e)
    }
}
