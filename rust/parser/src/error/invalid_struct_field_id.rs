use super::Error;
use crate::ast::{Ident, LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidStructFieldId {
    schema_name: String,
    id: LitPosInt,
    field_ident: Ident,
}

impl InvalidStructFieldId {
    pub(crate) fn validate(field: &StructField, validate: &mut Validate) {
        if field.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidStructFieldId {
            schema_name: validate.schema_name().to_owned(),
            id: field.id().clone(),
            field_ident: field.name().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn field_ident(&self) -> &Ident {
        &self.field_ident
    }
}

impl Diagnostic for InvalidStructFieldId {
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

impl From<InvalidStructFieldId> for Error {
    fn from(e: InvalidStructFieldId) -> Self {
        Error::InvalidStructFieldId(e)
    }
}
