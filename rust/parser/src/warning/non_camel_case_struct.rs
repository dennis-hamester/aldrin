use super::Warning;
use crate::ast::{Ident, StructDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use heck::CamelCase;

#[derive(Debug)]
pub struct NonCamelCaseStruct {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseStruct {
    pub(crate) fn validate(struct_def: &StructDef, validate: &mut Validate) {
        let camel_case = struct_def.name().value().to_camel_case();
        if struct_def.name().value() != camel_case {
            validate.add_warning(NonCamelCaseStruct {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: struct_def.name().clone(),
            });
        }
    }

    pub fn camel_case(&self) -> &str {
        &self.camel_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for NonCamelCaseStruct {
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

impl From<NonCamelCaseStruct> for Warning {
    fn from(w: NonCamelCaseStruct) -> Self {
        Warning::NonCamelCaseStruct(w)
    }
}
