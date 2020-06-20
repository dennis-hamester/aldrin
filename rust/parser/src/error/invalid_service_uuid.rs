use super::Error;
use crate::ast::{LitUuid, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct InvalidServiceUuid {
    schema_name: String,
    uuid: LitUuid,
}

impl InvalidServiceUuid {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        if !service_def.uuid().value().is_nil() {
            return;
        }

        validate.add_error(InvalidServiceUuid {
            schema_name: validate.schema_name().to_owned(),
            uuid: service_def.uuid().clone(),
        });
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }
}

impl Diagnostic for InvalidServiceUuid {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<InvalidServiceUuid> for Error {
    fn from(e: InvalidServiceUuid) -> Self {
        Error::InvalidServiceUuid(e)
    }
}
