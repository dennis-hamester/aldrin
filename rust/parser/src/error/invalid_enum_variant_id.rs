use super::Error;
use crate::ast::{EnumVariant, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidEnumVariantId {
    schema_name: String,
    id: LitPosInt,
    var_ident: Ident,
}

impl InvalidEnumVariantId {
    pub(crate) fn validate(var: &EnumVariant, validate: &mut Validate) {
        if var.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidEnumVariantId {
            schema_name: validate.schema_name().to_owned(),
            id: var.id().clone(),
            var_ident: var.name().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn variant_ident(&self) -> &Ident {
        &self.var_ident
    }
}

impl Diagnostic for InvalidEnumVariantId {
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

impl From<InvalidEnumVariantId> for Error {
    fn from(e: InvalidEnumVariantId) -> Self {
        Error::InvalidEnumVariantId(e)
    }
}
