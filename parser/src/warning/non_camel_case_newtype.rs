use super::Warning;
use crate::ast::{Ident, NewtypeDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToUpperCamelCase;

#[derive(Debug)]
pub struct NonCamelCaseNewtype {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseNewtype {
    pub(crate) fn validate(newtype_def: &NewtypeDef, validate: &mut Validate) {
        let camel_case = newtype_def.name().value().to_upper_camel_case();

        if newtype_def.name().value() != camel_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: newtype_def.name().clone(),
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

impl Diagnostic for NonCamelCaseNewtype {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "newtype `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming newtype `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));

        report.render()
    }
}

impl From<NonCamelCaseNewtype> for Warning {
    fn from(w: NonCamelCaseNewtype) -> Self {
        Self::NonCamelCaseNewtype(w)
    }
}
