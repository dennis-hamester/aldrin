use super::Error;
use crate::ast::{EnumVariant, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter, Renderer};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct InvalidEnumVariantId {
    schema_name: String,
    id: LitPosInt,
    var_ident: Ident,
}

impl InvalidEnumVariantId {
    pub(crate) fn validate(var: &EnumVariant, validate: &mut Validate) {
        if var.id().value().parse::<u32>().is_ok() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            id: var.id().clone(),
            var_ident: var.name().clone(),
        });
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn variant_ident(&self) -> &Ident {
        &self.var_ident
    }
}

impl Diagnostic for InvalidEnumVariantId {
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
                "invalid id `{}` for enum variant `{}`",
                self.id.value(),
                self.var_ident.value(),
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
            "invalid id `{}` for enum variant `{}`",
            self.id.value(),
            self.var_ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.id.span(), "id defined here");
        }

        report = report.help("ids must be u32 values in the range from 0 to 4294967295");
        report.render()
    }
}

impl From<InvalidEnumVariantId> for Error {
    fn from(e: InvalidEnumVariantId) -> Self {
        Self::InvalidEnumVariantId(e)
    }
}
