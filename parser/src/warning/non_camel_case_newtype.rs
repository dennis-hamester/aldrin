use super::{Warning, WarningKind};
use crate::ast::{Ident, NewtypeDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser};

#[derive(Debug)]
pub(crate) struct NonCamelCaseNewtype {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseNewtype {
    pub(crate) fn validate(newtype_def: &NewtypeDef, validate: &mut Validate) {
        if !Ident::is_valid(newtype_def.name().value()) {
            return;
        }

        let camel_case = util::to_camel_case(newtype_def.name().value());
        if newtype_def.name().value() == camel_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            camel_case,
            ident: newtype_def.name().clone(),
        });
    }
}

impl Diagnostic for NonCamelCaseNewtype {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "newtype `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
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
        Self {
            kind: WarningKind::NonCamelCaseNewtype(w),
        }
    }
}
