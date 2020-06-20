use super::Warning;
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;
use heck::CamelCase;

#[derive(Debug)]
pub struct NonCamelCaseService {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseService {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        let camel_case = service_def.name().value().to_camel_case();
        if service_def.name().value() != camel_case {
            validate.add_warning(NonCamelCaseService {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: service_def.name().clone(),
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

impl Diagnostic for NonCamelCaseService {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<NonCamelCaseService> for Warning {
    fn from(w: NonCamelCaseService) -> Self {
        Warning::NonCamelCaseService(w)
    }
}
