use super::Error;
use crate::ast::{Ident, LitPosInt, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidServiceVersion {
    schema_name: String,
    ver: LitPosInt,
    svc_ident: Ident,
}

impl InvalidServiceVersion {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        if service_def.version().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(InvalidServiceVersion {
            schema_name: validate.schema_name().to_owned(),
            ver: service_def.version().clone(),
            svc_ident: service_def.name().clone(),
        });
    }

    pub fn version(&self) -> &LitPosInt {
        &self.ver
    }

    pub fn service_ident(&self) -> &Ident {
        &self.svc_ident
    }
}

impl Diagnostic for InvalidServiceVersion {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::error(format!(
            "invalid version `{}` for service `{}`",
            self.ver.value(),
            self.svc_ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.ver.span().from,
                self.ver.span(),
                "version defined here",
            );
        }

        fmt.note("versions must be u32 values in the range from 0 to 4294967295");
        fmt.format()
    }
}

impl From<InvalidServiceVersion> for Error {
    fn from(e: InvalidServiceVersion) -> Self {
        Error::InvalidServiceVersion(e)
    }
}
