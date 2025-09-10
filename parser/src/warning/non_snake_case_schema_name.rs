use super::{Warning, WarningKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToSnakeCase;

#[derive(Debug)]
pub(crate) struct NonSnakeCaseSchemaName {
    schema_name: String,
    snake_case: String,
}

impl NonSnakeCaseSchemaName {
    pub(crate) fn validate(schema_name: &str, validate: &mut Validate) {
        let snake_case = schema_name.to_snake_case();
        if schema_name == snake_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: schema_name.to_owned(),
            snake_case,
        });
    }
}

impl Diagnostic for NonSnakeCaseSchemaName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, _parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "schema `{}` should have a snake-case name",
            self.schema_name
        ));

        report = report.help(format!(
            "consider renaming schema `{}` to `{}`",
            self.schema_name, self.snake_case
        ));

        report.render()
    }
}

impl From<NonSnakeCaseSchemaName> for Warning {
    fn from(w: NonSnakeCaseSchemaName) -> Self {
        Self {
            kind: WarningKind::NonSnakeCaseSchemaName(w),
        }
    }
}
