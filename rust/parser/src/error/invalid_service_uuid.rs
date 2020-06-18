use super::Error;
use crate::ast::{LitUuid, ServiceDef};
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

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }
}

impl From<InvalidServiceUuid> for Error {
    fn from(e: InvalidServiceUuid) -> Self {
        Error::InvalidServiceUuid(e)
    }
}
