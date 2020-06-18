use super::Error;
use crate::ast::{LitPosInt, ServiceDef};
use crate::validate::Validate;

#[derive(Debug)]
pub struct InvalidServiceVersion {
    schema_name: String,
    ver: LitPosInt,
}

impl InvalidServiceVersion {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        if service_def.version().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidServiceVersion {
            schema_name: validate.schema_name().to_owned(),
            ver: service_def.version().clone(),
        });
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn version(&self) -> &LitPosInt {
        &self.ver
    }
}

impl From<InvalidServiceVersion> for Error {
    fn from(e: InvalidServiceVersion) -> Self {
        Error::InvalidServiceVersion(e)
    }
}
