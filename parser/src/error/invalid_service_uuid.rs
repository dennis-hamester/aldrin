use super::Error;
use crate::ast::{Ident, LitUuid, ServiceDef};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
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

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "invalid uuid `{}` for service `{}`",
                Uuid::nil(),
                self.svc_ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.uuid.span().from, self.uuid.span(), "nil uuid");
        }

        fmt.note("the nil uuid cannot be used for services");
        fmt.help(format!("use e.g. `{}`", Uuid::new_v4()));

        fmt.format()
    }
}

impl From<InvalidServiceUuid> for Error {
    fn from(e: InvalidServiceUuid) -> Self {
        Self::InvalidServiceUuid(e)
    }
}
