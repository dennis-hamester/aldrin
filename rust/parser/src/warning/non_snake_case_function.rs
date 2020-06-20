use super::Warning;
use crate::ast::{FunctionDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use heck::SnakeCase;

#[derive(Debug)]
pub struct NonSnakeCaseFunction {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseFunction {
    pub(crate) fn validate(func: &FunctionDef, validate: &mut Validate) {
        let snake_case = func.name().value().to_snake_case();
        if func.name().value() != snake_case {
            validate.add_warning(NonSnakeCaseFunction {
                schema_name: validate.schema_name().to_owned(),
                snake_case,
                ident: func.name().clone(),
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

impl Diagnostic for NonSnakeCaseFunction {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        todo!()
    }
}

impl From<NonSnakeCaseFunction> for Warning {
    fn from(w: NonSnakeCaseFunction) -> Self {
        Warning::NonSnakeCaseFunction(w)
    }
}
