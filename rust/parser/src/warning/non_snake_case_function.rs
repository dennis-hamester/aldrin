use super::Warning;
use crate::ast::{FunctionDef, Ident};
use crate::validate::Validate;
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

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn snake_case(&self) -> &str {
        &self.snake_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl From<NonSnakeCaseFunction> for Warning {
    fn from(w: NonSnakeCaseFunction) -> Self {
        Warning::NonSnakeCaseFunction(w)
    }
}
