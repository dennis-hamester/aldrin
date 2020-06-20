use super::Error;
use crate::ast::SchemaName;
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct MissingImport {
    schema_name: String,
    extern_schema: SchemaName,
}

impl MissingImport {
    pub(crate) fn validate(schema_name: &SchemaName, validate: &mut Validate) {
        for import in validate.get_current_schema().imports() {
            if import.schema_name().value() == schema_name.value() {
                return;
            }
        }

        validate.add_error(MissingImport {
            schema_name: validate.schema_name().to_owned(),
            extern_schema: schema_name.clone(),
        });
    }

    pub fn extern_schema(&self) -> &SchemaName {
        &self.extern_schema
    }
}

impl Diagnostic for MissingImport {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<MissingImport> for Error {
    fn from(e: MissingImport) -> Self {
        Error::MissingImport(e)
    }
}
