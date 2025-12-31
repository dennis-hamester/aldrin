use super::{Warning, WarningKind};
use crate::ast::{EnumDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, util};

#[derive(Debug)]
pub(crate) struct NonCamelCaseEnum {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnum {
    pub(crate) fn validate(enum_def: &EnumDef, validate: &mut Validate) {
        if !Ident::is_valid(enum_def.name().value()) {
            return;
        }

        let camel_case = util::to_camel_case(enum_def.name().value());
        if enum_def.name().value() == camel_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            camel_case,
            ident: enum_def.name().clone(),
        });
    }
}

impl Diagnostic for NonCamelCaseEnum {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "enum `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
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
        Self {
            kind: WarningKind::NonCamelCaseEnum(w),
        }
    }
}
