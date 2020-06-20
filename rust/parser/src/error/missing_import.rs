use super::Error;
use crate::ast::SchemaName;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed};

#[derive(Debug)]
pub struct MissingImport {
    schema_name: String,
    extern_schema: SchemaName,
    candidate: Option<String>,
}

impl MissingImport {
    pub(crate) fn validate(schema_name: &SchemaName, validate: &mut Validate) {
        let schema = validate.get_current_schema();
        for import in schema.imports() {
            if import.schema_name().value() == schema_name.value() {
                return;
            }
        }

        let candidates = schema.imports().iter().map(|i| i.schema_name().value());
        let candidate = util::did_you_mean(candidates, schema_name.value()).map(ToOwned::to_owned);

        validate.add_error(MissingImport {
            schema_name: validate.schema_name().to_owned(),
            extern_schema: schema_name.clone(),
            candidate,
        });
    }

    pub fn extern_schema(&self) -> &SchemaName {
        &self.extern_schema
    }

    pub fn candidate(&self) -> Option<&str> {
        self.candidate.as_deref()
    }
}

impl Diagnostic for MissingImport {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::error(format!("missing import `{}`", self.extern_schema.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.extern_schema.span().from,
                self.extern_schema.span(),
                format!("schema `{}` used here", self.extern_schema.value()),
            );
        }

        if let Some(ref candidate) = self.candidate {
            fmt.help(format!("did you mean `{}`?", candidate));
        } else {
            fmt.help(format!(
                "add `import {};` at the top",
                self.extern_schema.value()
            ));
        };

        fmt.format()
    }
}

impl From<MissingImport> for Error {
    fn from(e: MissingImport) -> Self {
        Error::MissingImport(e)
    }
}
