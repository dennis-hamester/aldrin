use super::Warning;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToUpperCamelCase;

#[derive(Debug)]
pub struct NonCamelCaseEnumVariant {
    schema_name: String,
    camel_case: String,
    ident: Ident,
}

impl NonCamelCaseEnumVariant {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        let camel_case = ident.value().to_upper_camel_case();

        if ident.value() != camel_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                camel_case,
                ident: ident.clone(),
            });
        }
    }

    pub fn camel_case(&self) -> &str {
        &self.camel_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for NonCamelCaseEnumVariant {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "variant `{}` should have a camel-case name",
                self.ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident.span().from, self.ident.span(), "");
        }

        fmt.help(format!(
            "consider renaming variant `{}` to `{}`",
            self.ident.value(),
            self.camel_case
        ));
        fmt.format()
    }
}

impl From<NonCamelCaseEnumVariant> for Warning {
    fn from(w: NonCamelCaseEnumVariant) -> Self {
        Self::NonCamelCaseEnumVariant(w)
    }
}
