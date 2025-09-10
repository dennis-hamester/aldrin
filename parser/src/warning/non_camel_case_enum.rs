use super::Warning;
use crate::ast::{EnumDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToUpperCamelCase;

#[derive(Debug)]
pub struct NonCamelCaseEnum {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnum {
    pub(crate) fn validate(enum_def: &EnumDef, validate: &mut Validate) {
        let camel_case = enum_def.name().value().to_upper_camel_case();
        if enum_def.name().value() != camel_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: enum_def.name().clone(),
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

impl Diagnostic for NonCamelCaseEnum {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "enum `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming enum `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));

        report.render()
    }
}

impl From<NonCamelCaseEnum> for Warning {
    fn from(w: NonCamelCaseEnum) -> Self {
        Self::NonCamelCaseEnum(w)
    }
}
