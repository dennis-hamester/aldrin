use super::{Error, ErrorKind};
use crate::ast::{Ident, LitPosInt, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub(crate) struct InvalidServiceVersion {
    schema_name: String,
    ver: LitPosInt,
    svc_ident: Ident,
}

impl InvalidServiceVersion {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        if service_def.version().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ver: service_def.version().clone(),
            svc_ident: service_def.name().clone(),
        });
    }
}

impl Diagnostic for InvalidServiceVersion {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(format!(
            "invalid version `{}` for service `{}`",
            self.ver.value(),
            self.svc_ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ver.span(), "version defined here");
        }

        report = report.note("versions must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidServiceVersion> for Error {
    fn from(e: InvalidServiceVersion) -> Self {
        Self {
            kind: ErrorKind::InvalidServiceVersion(e),
        }
    }
}
