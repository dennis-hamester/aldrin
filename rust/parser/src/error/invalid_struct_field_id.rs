use super::Error;
use crate::ast::{LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct InvalidStructFieldId {
    schema_name: String,
    id: LitPosInt,
}

impl InvalidStructFieldId {
    pub(crate) fn validate(field: &StructField, validate: &mut Validate) {
        if field.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidStructFieldId {
            schema_name: validate.schema_name().to_owned(),
            id: field.id().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }
}

impl Diagnostic for InvalidStructFieldId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<InvalidStructFieldId> for Error {
    fn from(e: InvalidStructFieldId) -> Self {
        Error::InvalidStructFieldId(e)
    }
}
