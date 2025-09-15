use super::{Warning, WarningKind};
use crate::ast::{FunctionDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser};

#[derive(Debug)]
pub(crate) struct NonSnakeCaseFunction {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseFunction {
    pub(crate) fn validate(func: &FunctionDef, validate: &mut Validate) {
        if !Ident::is_valid(func.name().value()) {
            return;
        }

        let snake_case = util::to_snake_case(func.name().value());
        if func.name().value() == snake_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            snake_case,
            ident: func.name().clone(),
        });
    }
}

impl Diagnostic for NonSnakeCaseFunction {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "function `{}` should have a snake-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming function `{}` to `{}`",
            self.ident.value(),
            self.snake_case
        ));

        report.render()
    }
}

impl From<NonSnakeCaseFunction> for Warning {
    fn from(w: NonSnakeCaseFunction) -> Self {
        Self {
            kind: WarningKind::NonSnakeCaseFunction(w),
        }
    }
}
