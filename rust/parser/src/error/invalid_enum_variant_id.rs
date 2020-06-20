use super::Error;
use crate::ast::{EnumVariant, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct InvalidEnumVariantId {
    schema_name: String,
    id: LitPosInt,
}

impl InvalidEnumVariantId {
    pub(crate) fn validate(var: &EnumVariant, validate: &mut Validate) {
        if var.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidEnumVariantId {
            schema_name: validate.schema_name().to_owned(),
            id: var.id().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }
}

impl Diagnostic for InvalidEnumVariantId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<InvalidEnumVariantId> for Error {
    fn from(e: InvalidEnumVariantId) -> Self {
        Error::InvalidEnumVariantId(e)
    }
}
