use super::Error;
use crate::ast::{EventDef, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidEventId {
    schema_name: String,
    id: LitPosInt,
    name_ident: Ident,
}

impl InvalidEventId {
    pub(crate) fn validate(func: &EventDef, validate: &mut Validate) {
        if func.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: func.id().clone(),
            name_ident: func.name().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn name_ident(&self) -> &Ident {
        &self.name_ident
    }
}

impl Diagnostic for InvalidEventId {
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
                "invalid id `{}` for event `{}`",
                self.id.value(),
                self.name_ident.value(),
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.id.span().from,
                self.id.span(),
                "id defined here",
            );
        }

        fmt.help("ids must be u32 values in the range from 0 to 4294967295");
        fmt.format()
    }
}

impl From<InvalidEventId> for Error {
    fn from(e: InvalidEventId) -> Self {
        Error::InvalidEventId(e)
    }
}
