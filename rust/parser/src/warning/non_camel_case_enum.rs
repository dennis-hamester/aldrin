use super::Warning;
use crate::ast::{EnumDef, Ident};
use crate::validate::Validate;
use heck::CamelCase;

#[derive(Debug)]
pub struct NonCamelCaseEnum {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnum {
    pub(crate) fn validate(enum_def: &EnumDef, validate: &mut Validate) {
        let camel_case = enum_def.name().value().to_camel_case();
        if enum_def.name().value() != camel_case {
            validate.add_warning(NonCamelCaseEnum {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: enum_def.name().clone(),
            });
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn camel_case(&self) -> &str {
        &self.camel_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl From<NonCamelCaseEnum> for Warning {
    fn from(w: NonCamelCaseEnum) -> Self {
        Warning::NonCamelCaseEnum(w)
    }
}
