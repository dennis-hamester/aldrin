use super::Warning;
use crate::ast::{ConstDef, Ident};
use crate::validate::Validate;
use heck::ShoutySnakeCase;

#[derive(Debug)]
pub struct NonShoutySnakeCaseConst {
    schema_name: String,
    shouty_snake_case: String,
    ident: Ident,
}

impl NonShoutySnakeCaseConst {
    pub(crate) fn validate(const_def: &ConstDef, validate: &mut Validate) {
        let shouty_snake_case = const_def.name().value().to_shouty_snake_case();
        if const_def.name().value() != shouty_snake_case {
            validate.add_warning(NonShoutySnakeCaseConst {
                schema_name: validate.schema_name().to_owned(),
                shouty_snake_case,
                ident: const_def.name().clone(),
            });
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn shouty_snake_case(&self) -> &str {
        &self.shouty_snake_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl From<NonShoutySnakeCaseConst> for Warning {
    fn from(w: NonShoutySnakeCaseConst) -> Self {
        Warning::NonShoutySnakeCaseConst(w)
    }
}