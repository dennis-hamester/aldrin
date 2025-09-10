use super::Error;
use crate::ast::{Ident, LitUuid, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use uuid::Uuid;

#[derive(Debug)]
pub struct InvalidServiceUuid {
    schema_name: String,
    uuid: LitUuid,
    svc_ident: Ident,
}

impl InvalidServiceUuid {
    pub(crate) fn validate(service_def: &ServiceDef, validate: &mut Validate) {
        if !service_def.uuid().value().is_nil() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            uuid: service_def.uuid().clone(),
            svc_ident: service_def.name().clone(),
        });
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }

    pub fn service_ident(&self) -> &Ident {
        &self.svc_ident
    }
}

impl Diagnostic for InvalidServiceUuid {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(format!(
            "invalid uuid `{}` for service `{}`",
            Uuid::nil(),
            self.svc_ident.value()
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.uuid.span(), "nil uuid");
        }

        report = report.note("the nil uuid cannot be used for services");
        report.render()
    }
}

impl From<InvalidServiceUuid> for Error {
    fn from(e: InvalidServiceUuid) -> Self {
        Self::InvalidServiceUuid(e)
    }
}
