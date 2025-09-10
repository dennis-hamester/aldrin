use super::Error;
use crate::ast::{Ident, LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter, Renderer};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidStructFieldId {
    schema_name: String,
    id: LitPosInt,
    field_ident: Ident,
}

impl InvalidStructFieldId {
    pub(crate) fn validate(field: &StructField, validate: &mut Validate) {
        if field.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: field.id().clone(),
            field_ident: field.name().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn field_ident(&self) -> &Ident {
        &self.field_ident
    }
}

impl Diagnostic for InvalidStructFieldId {
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
                "invalid id `{}` for struct field `{}`",
                self.id.value(),
                self.field_ident.value(),
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

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(format!(
            "invalid id `{}` for struct field `{}`",
            self.id.value(),
            self.field_ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.id.span(), "id defined here");
        }

        report = report.help("ids must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidStructFieldId> for Error {
    fn from(e: InvalidStructFieldId) -> Self {
        Self::InvalidStructFieldId(e)
    }
}
