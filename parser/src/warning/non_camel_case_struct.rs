use super::{Warning, WarningKind};
use crate::ast::{Ident, StructDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parser;
use heck::ToUpperCamelCase;

#[derive(Debug)]
pub(crate) struct NonCamelCaseStruct {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseStruct {
    pub(crate) fn validate(struct_def: &StructDef, validate: &mut Validate) {
        let camel_case = struct_def.name().value().to_upper_camel_case();
        if struct_def.name().value() != camel_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: struct_def.name().clone(),
            });
        }
    }
}

impl Diagnostic for NonCamelCaseStruct {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "struct `{}` should have a camel-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming struct `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));

        report.render()
    }
}

impl From<NonCamelCaseStruct> for Warning {
    fn from(w: NonCamelCaseStruct) -> Self {
        Self {
            kind: WarningKind::NonCamelCaseStruct(w),
        }
    }
}
