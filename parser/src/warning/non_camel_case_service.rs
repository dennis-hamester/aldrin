use super::Warning;
use crate::ast::{Ident, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToUpperCamelCase;

#[derive(Debug)]
pub struct NonCamelCaseService {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseService {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        let camel_case = service_def.name().value().to_upper_camel_case();
        if service_def.name().value() != camel_case {
            validate.add_warning(Self {
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

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "service `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming service `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));

        report.render()
    }
}

impl From<NonCamelCaseService> for Warning {
    fn from(w: NonCamelCaseService) -> Self {
        Self::NonCamelCaseService(w)
    }
}
