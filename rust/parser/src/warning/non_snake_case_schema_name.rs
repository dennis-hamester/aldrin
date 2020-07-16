use super::Warning;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use heck::SnakeCase;

#[derive(Debug)]
pub struct NonSnakeCaseSchemaName {
    schema_name: String,
    snake_case: String,
}

impl NonSnakeCaseSchemaName {
    pub(crate) fn validate(schema_name: &str, validate: &mut Validate) {
        let snake_case = schema_name.to_snake_case();
        if schema_name == snake_case {
            return;
        }

        validate.add_warning(NonSnakeCaseSchemaName {
            schema_name: schema_name.to_owned(),
            snake_case,
        });
    }

    pub fn snake_case(&self) -> &str {
        &self.snake_case
    }
}

impl Diagnostic for NonSnakeCaseSchemaName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, _parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "schema `{}` should have a snake-case name",
                self.schema_name
            ),
        );

        fmt.help(format!(
            "consider renaming schema `{}` to `{}`",
            self.schema_name, self.snake_case
        ));

        fmt.format()
    }
}

impl From<NonSnakeCaseSchemaName> for Warning {
    fn from(w: NonSnakeCaseSchemaName) -> Self {
        Warning::NonSnakeCaseSchemaName(w)
    }
}
