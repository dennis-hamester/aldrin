use super::Warning;
use crate::ast::{EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;
use heck::CamelCase;

#[derive(Debug)]
pub struct NonCamelCaseEnumVariant {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnumVariant {
    pub(crate) fn validate(var: &EnumVariant, validate: &mut Validate) {
        let camel_case = var.name().value().to_camel_case();
        if var.name().value() != camel_case {
            validate.add_warning(NonCamelCaseEnumVariant {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: var.name().clone(),
            });
        }
    }

    pub fn camel_case(&self) -> &str {
        &self.camel_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for NonCamelCaseEnumVariant {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<NonCamelCaseEnumVariant> for Warning {
    fn from(w: NonCamelCaseEnumVariant) -> Self {
        Warning::NonCamelCaseEnumVariant(w)
    }
}
