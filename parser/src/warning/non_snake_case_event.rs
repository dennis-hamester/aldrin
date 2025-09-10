use super::Warning;
use crate::ast::{EventDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToSnakeCase;

#[derive(Debug)]
pub struct NonSnakeCaseEvent {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseEvent {
    pub(crate) fn validate(ev: &EventDef, validate: &mut Validate) {
        let snake_case = ev.name().value().to_snake_case();
        if ev.name().value() != snake_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                snake_case,
                ident: ev.name().clone(),
            });
        }
    }

    pub fn snake_case(&self) -> &str {
        &self.snake_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for NonSnakeCaseEvent {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "event `{}` should have a snake-case name",
                self.ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident.span().from, self.ident.span(), "");
        }

        fmt.help(format!(
            "consider renaming event `{}` to `{}`",
            self.ident.value(),
            self.snake_case
        ));
        fmt.format()
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "event `{}` should have a snake-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
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
        Self::NonSnakeCaseEvent(w)
    }
}
