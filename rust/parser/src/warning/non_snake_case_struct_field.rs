use super::Warning;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;
use heck::SnakeCase;

#[derive(Debug)]
pub struct NonSnakeCaseStructField {
    schema_name: String,
    snake_case: String,
    ident: Ident,
}

impl NonSnakeCaseStructField {
    pub(crate) fn validate(struct_field: &StructField, validate: &mut Validate) {
        let snake_case = struct_field.name().value().to_snake_case();
        if struct_field.name().value() != snake_case {
            validate.add_warning(NonSnakeCaseStructField {
                schema_name: validate.schema_name().to_owned(),
                snake_case,
                ident: struct_field.name().clone(),
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

impl Diagnostic for NonSnakeCaseStructField {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<NonSnakeCaseStructField> for Warning {
    fn from(w: NonSnakeCaseStructField) -> Self {
        Warning::NonSnakeCaseStructField(w)
    }
}
