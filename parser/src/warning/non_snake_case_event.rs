use super::{Warning, WarningKind};
use crate::ast::{EventDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parser;
use heck::ToSnakeCase;

#[derive(Debug)]
pub(crate) struct NonSnakeCaseEvent {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseEvent {
    pub(crate) fn validate(ev: &EventDef, validate: &mut Validate) {
        if !Ident::is_valid(ev.name().value()) {
            return;
        }

        let snake_case = ev.name().value().to_snake_case();
        if ev.name().value() == snake_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            snake_case,
            ident: ev.name().clone(),
        });
    }
}

impl Diagnostic for NonSnakeCaseEvent {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "event `{}` should have a snake-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming event `{}` to `{}`",
            self.ident.value(),
            self.snake_case
        ));

        report.render()
    }
}

impl From<NonSnakeCaseEvent> for Warning {
    fn from(w: NonSnakeCaseEvent) -> Self {
        Self {
            kind: WarningKind::NonSnakeCaseEvent(w),
        }
    }
}
