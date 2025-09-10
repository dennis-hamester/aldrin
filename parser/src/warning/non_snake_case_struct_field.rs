use super::{Warning, WarningKind};
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToSnakeCase;

#[derive(Debug)]
pub(crate) struct NonSnakeCaseStructField {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseStructField {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        let snake_case = ident.value().to_snake_case();

        if ident.value() != snake_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                snake_case,
                ident: ident.clone(),
            });
        }
    }
}

impl Diagnostic for NonSnakeCaseStructField {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "field `{}` should have a snake-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming field `{}` to `{}`",
            self.ident.value(),
            self.snake_case
        ));

        report.render()
    }
}

impl From<NonSnakeCaseStructField> for Warning {
    fn from(w: NonSnakeCaseStructField) -> Self {
        Self {
            kind: WarningKind::NonSnakeCaseStructField(w),
        }
    }
}
