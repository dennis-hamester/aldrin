use super::Error;
use crate::ast::{Ident, SchemaName};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct ExternTypeNotFound {
    schema_name: String,
    extern_schema: SchemaName,
    extern_ident: Ident,
}

impl ExternTypeNotFound {
    pub(crate) fn validate(schema_name: &SchemaName, ident: &Ident, validate: &mut Validate) {
        let schema = match validate.get_schema(schema_name.value()) {
            Some(schema) => schema,
            None => return,
        };

        for def in schema.definitions() {
            if def.name().value() == ident.value() {
                return;
            }
        }

        validate.add_error(ExternTypeNotFound {
            schema_name: validate.schema_name().to_owned(),
            extern_schema: schema_name.clone(),
            extern_ident: ident.clone(),
        });
    }

    pub fn extern_schema(&self) -> &SchemaName {
        &self.extern_schema
    }

    pub fn extern_ident(&self) -> &Ident {
        &self.extern_ident
    }
}

impl Diagnostic for ExternTypeNotFound {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        todo!()
    }
}

impl From<ExternTypeNotFound> for Error {
    fn from(e: ExternTypeNotFound) -> Self {
        Error::ExternTypeNotFound(e)
    }
}
