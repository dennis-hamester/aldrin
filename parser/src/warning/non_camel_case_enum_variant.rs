use super::{Warning, WarningKind};
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, util};

#[derive(Debug)]
pub(crate) struct NonCamelCaseEnumVariant {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnumVariant {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        if !Ident::is_valid(ident.value()) {
            return;
        }

        let camel_case = util::to_camel_case(ident.value());
        if ident.value() == camel_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            camel_case,
            ident: ident.clone(),
        });
    }
}

impl Diagnostic for NonCamelCaseEnumVariant {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "variant `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming variant `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));

        report.render()
    }
}

impl From<NonCamelCaseEnumVariant> for Warning {
    fn from(w: NonCamelCaseEnumVariant) -> Self {
        Self {
            kind: WarningKind::NonCamelCaseEnumVariant(w),
        }
    }
}
